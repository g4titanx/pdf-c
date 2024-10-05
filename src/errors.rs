use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CompressionError {
    PdfLoadError(lopdf::Error),
    PdfSaveError(lopdf::Error),
    ImageLoadError(image::ImageError),
    ImageSaveError(image::ImageError),
    IoError(std::io::Error),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompressionError::PdfLoadError(e) => write!(f, "Failed to load PDF: {}", e),
            CompressionError::PdfSaveError(e) => write!(f, "Failed to save PDF: {}", e),
            CompressionError::ImageLoadError(e) => write!(f, "Failed to load image: {}", e),
            CompressionError::ImageSaveError(e) => write!(f, "Failed to save image: {}", e),
            CompressionError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl Error for CompressionError {}

impl From<lopdf::Error> for CompressionError {
    fn from(err: lopdf::Error) -> CompressionError {
        CompressionError::PdfLoadError(err)
    }
}

impl From<image::ImageError> for CompressionError {
    fn from(err: image::ImageError) -> CompressionError {
        CompressionError::ImageLoadError(err)
    }
}

impl From<std::io::Error> for CompressionError {
    fn from(err: std::io::Error) -> CompressionError {
        CompressionError::IoError(err)
    }
}
