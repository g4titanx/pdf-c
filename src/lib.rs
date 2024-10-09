mod errors;

use errors::CompressionError;
use image::{DynamicImage, ImageOutputFormat};
use log::{info, warn};
use lopdf::{Dictionary, Document, Object};
use miniz_oxide::deflate::compress_to_vec_zlib;
use sgxkit::io::{get_input_data_string, OutputWriter};
use std::io::{Cursor, Write};

#[no_mangle]
pub extern "C" fn main() {
    let input = sgxkit::io::get_input_data_string().unwrap_or_else(|e| format!("Error reading input: {:?}", e));
    let mut writer = sgxkit::io::OutputWriter::new();
    writer.write_all(format!("Received input: {}", input).as_bytes()).unwrap();
    
    match get_input_data_string() {
        Ok(input) => {
            match compress_pdf(input.as_bytes()) {
                Ok(compressed) => {
                    if let Err(e) = writer.write_all(&compressed) {
                        let _ = writer.write_all(format!("Failed to write compressed PDF: {:?}\n", e).as_bytes());
                    } else {
                        let _ = writer.write_all(b"PDF compression successful\n");
                    }
                }
                Err(e) => {
                    let _ = writer.write_all(format!("Compression error: {:?}\n", e).as_bytes());
                }
            }
        }
        Err(e) => {
            let _ = writer.write_all(format!("Failed to read input: {:?}\n", e).as_bytes());
        }
    }
}

fn compress_pdf(input: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut compressor = PdfCompressor::new();
    compressor.compress(input)
}
pub struct PdfCompressor;

impl PdfCompressor {
    pub fn new() -> Self {
        PdfCompressor
    }

    pub fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut doc = Document::load_from(input).map_err(|e| {
            println!("PDF load error: {:?}", e);
            CompressionError::PdfLoad(e)
        })?;
        info!("PDF loaded successfully");

        self.compress_images(&mut doc)?;
        self.compress_text_streams(&mut doc)?;
        self.remove_metadata(&mut doc);

        let mut output = Vec::new();
        doc.save_to(&mut output)?;

        if output.len() >= input.len() {
            warn!(
                "Compression was ineffective. Output size: {}, Input size: {}",
                output.len(),
                input.len()
            );
        } else {
            info!(
                "PDF compressed successfully. Size reduced from {} to {} bytes",
                input.len(),
                output.len()
            );
        }

        Ok(output)
    }

    fn compress_images(&self, doc: &mut Document) -> Result<(), CompressionError> {
        for (_id, object) in doc.objects.iter_mut() {
            if let Object::Stream(ref mut stream) = object {
                let filter = match stream.dict.get(b"Filter") {
                    Ok(Object::Name(filter)) => filter.clone(),
                    _ => continue,
                };

                let original_size = stream.content.len();
                let image_data = stream.content.clone();
                stream.decompress();

                match filter.as_slice() {
                    b"DCTDecode" => {
                        // Recompress JPEG with lower quality
                        if let Ok(img) = image::load_from_memory(&image_data) {
                            let compressed_img = self.compress_jpeg(img, 70)?; // Adjust quality as needed
                            if compressed_img.len() < original_size {
                                let reduction = (1.0
                                    - (compressed_img.len() as f64 / original_size as f64))
                                    * 100.0;
                                info!("JPEG recompressed. Reduction: {:.2}%", reduction);
                                stream.set_content(compressed_img);
                            } else {
                                info!("JPEG recompression ineffective. Keeping original.");
                            }
                        }
                    }
                    b"FlateDecode" | b"JPXDecode" | _ => {
                        // Try PNG compression for other formats
                        if let Ok(img) = image::load_from_memory(&image_data) {
                            let compressed_img = self.compress_png(img)?;
                            if compressed_img.len() < original_size {
                                let reduction = (1.0
                                    - (compressed_img.len() as f64 / original_size as f64))
                                    * 100.0;
                                info!("Image compressed to PNG. Reduction: {:.2}%", reduction);
                                stream.set_content(compressed_img);
                                stream
                                    .dict
                                    .set("Filter", Object::Name(b"FlateDecode".to_vec()));
                            } else {
                                info!("PNG compression ineffective. Keeping original.");
                            }
                        } else {
                            warn!("Unable to load image. Skipping.");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn compress_jpeg(&self, img: DynamicImage, quality: u8) -> Result<Vec<u8>, CompressionError> {
        let mut output = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut output),
            ImageOutputFormat::Jpeg(quality),
        )
        .map_err(CompressionError::ImageSave)?;
        Ok(output)
    }

    fn compress_png(&self, img: DynamicImage) -> Result<Vec<u8>, CompressionError> {
        let mut output = Vec::new();
        img.write_to(&mut Cursor::new(&mut output), ImageOutputFormat::Png)
            .map_err(CompressionError::ImageSave)?;
        Ok(output)
    }

    fn compress_text_streams(&self, doc: &mut Document) -> Result<(), CompressionError> {
        for (_id, object) in doc.objects.iter_mut() {
            if let Object::Stream(ref mut stream) = object {
                if stream.dict.get(b"Filter").is_err() {
                    let original_size = stream.content.len();
                    // Use 9 for best compression (0-9 scale)
                    let compressed = compress_to_vec_zlib(&stream.content, 9);

                    if compressed.len() < original_size {
                        let compression_ratio =
                            (1.0 - (compressed.len() as f64 / original_size as f64)) * 100.0;
                        info!(
                            "Text stream compressed. Reduction: {:.2}%",
                            compression_ratio
                        );
                        stream.set_content(compressed);
                        stream
                            .dict
                            .set("Filter", Object::Name(b"FlateDecode".to_vec()));
                    } else {
                        warn!("Text stream compression ineffective. Keeping original.");
                    }
                }
            }
        }
        Ok(())
    }

    fn remove_metadata(&self, doc: &mut Document) {
        if let Ok(Object::Dictionary(ref mut dict)) = doc.trailer.get_mut(b"Info") {
            info!("Removing metadata");
            *dict = Dictionary::new();
            info!("Metadata removed successfully");
        } else {
            warn!("No metadata found to remove");
        }
    }
}

// Run test with: cargo test -- --nocapture
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compression() {
        let mut compressor = PdfCompressor::new();
        let input_path = "test_files/smp1.pdf";

        println!("Reading input file...");
        let input = match fs::read(input_path) {
            Ok(data) => data,
            Err(e) => panic!("Failed to read test PDF: {}", e),
        };

        println!("Input PDF size: {} bytes", input.len());

        println!("Starting compression...");
        let result = compressor.compress(&input);

        match result {
            Ok(compressed) => {
                println!("Compression completed.");
                println!("Compressed PDF size: {} bytes", compressed.len());

                let compression_ratio = compressed.len() as f64 / input.len() as f64;
                println!("Compression ratio: {:.2}%", compression_ratio * 100.0);

                if compressed.len() < input.len() {
                    println!(
                        "Compression successful. Size reduced by {} bytes",
                        input.len() - compressed.len()
                    );
                    fs::write("test_files/compressed_sample.pdf", &compressed)
                        .expect("Failed to write compressed PDF");
                } else {
                    println!("Warning: Compressed file is larger than input.");
                }

                assert!(compressed.len() > 0, "Compressed data is empty");
            }
            Err(e) => {
                panic!("Compression failed: {:?}", e);
            }
        }
    }
}
