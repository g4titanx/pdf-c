use pdf_c::*;
use sgxkit::io::{get_input_data_string, OutputWriter};
use std::io::Write;

#[no_mangle]
pub extern "C" fn compress_pdf() {
    let input = get_input_data_string().unwrap_or_else(|_| "invalid data".to_string());
    let mut writer = OutputWriter::new();

    let mut compressor = PdfCompressor::new();

    match compressor.compress(input.as_bytes()) {
        Ok(compressed) => {
            writer
                .write_all(&compressed)
                .unwrap_or_else(|e| eprintln!("Failed to write compressed PDF: {:?}", e));
            writer
                .write_all(b"PDF compression successful\n")
                .unwrap_or_else(|e| eprintln!("Failed to write success message: {:?}", e));
        }
        Err(e) => {
            writer
                .write_all(format!("Compression error: {:?}\n", e).as_bytes())
                .unwrap_or_else(|e| eprintln!("Failed to write error message: {:?}", e));
        }
    }
}

fn main() {
    compress_pdf();
}
