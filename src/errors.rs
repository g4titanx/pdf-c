use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CompressionError {
    PdfLoad(lopdf::Error),
    PdfSave(lopdf::Error),
    ImageLoad(image::ImageError),
    ImageSave(image::ImageError),
    Io(std::io::Error),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompressionError::PdfLoad(e) => write!(f, "Failed to load PDF: {}", e),
            CompressionError::PdfSave(e) => write!(f, "Failed to save PDF: {}", e),
            CompressionError::ImageLoad(e) => write!(f, "Failed to load image: {}", e),
            CompressionError::ImageSave(e) => write!(f, "Failed to save image: {}", e),
            CompressionError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl Error for CompressionError {}

impl From<lopdf::Error> for CompressionError {
    fn from(err: lopdf::Error) -> CompressionError {
        CompressionError::PdfLoad(err)
    }
}

impl From<image::ImageError> for CompressionError {
    fn from(err: image::ImageError) -> CompressionError {
        CompressionError::ImageLoad(err)
    }
}

impl From<std::io::Error> for CompressionError {
    fn from(err: std::io::Error) -> CompressionError {
        CompressionError::Io(err)
    }
}
