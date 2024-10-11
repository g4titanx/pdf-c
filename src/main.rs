use base64::{engine::general_purpose, Engine as _};
use pdf_c::*;
use sgxkit::io::{get_input_data_string, OutputWriter};
use std::io::Write;

fn main() {
    let input = get_input_data_string().unwrap_or_else(|_| "invalid data".to_string());
    let mut writer = OutputWriter::new();

    // debug: print first 100 characters of the input
    writer
        .write_all(
            format!(
                "Received input (first 100 chars): {}\n",
                &input[..100.min(input.len())]
            )
            .as_bytes(),
        )
        .unwrap_or_else(|e| eprintln!("Failed to write debug info: {:?}", e));

    let decoded_input = match general_purpose::STANDARD.decode(input) {
        Ok(decoded) => decoded,
        Err(e) => {
            writer
                .write_all(format!("Failed to decode base64: {:?}\n", e).as_bytes())
                .unwrap_or_else(|e| eprintln!("Failed to write error message: {:?}", e));
            return;
        }
    };

    // Debug: Print the size of the decoded input
    writer
        .write_all(format!("Decoded input size: {} bytes\n", decoded_input.len()).as_bytes())
        .unwrap_or_else(|e| eprintln!("Failed to write debug info: {:?}", e));

    let mut compressor = PdfCompressor::new();

    match compressor.compress(&decoded_input) {
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
