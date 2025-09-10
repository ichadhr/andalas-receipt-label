use thiserror::Error;
use std::io;
use serde_json;
use qrcode::types;
use image;

/// Simplified error types for the ShipLabel library
#[derive(Error, Debug)]
pub enum ShipLabelError {
    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("Font error: {0}")]
    Font(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Data error: {0}")]
    Data(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

/// Result type alias for ShipLabel operations
pub type ShipLabelResult<T> = Result<T, ShipLabelError>;

/// Convert krilla errors to ShipLabelError
impl From<krilla::error::KrillaError> for ShipLabelError {
    fn from(err: krilla::error::KrillaError) -> Self {
        ShipLabelError::Pdf(format!("Krilla PDF error: {:?}", err))
    }
}

/// Convert std::io::Error to ShipLabelError
impl From<io::Error> for ShipLabelError {
    fn from(err: io::Error) -> Self {
        ShipLabelError::Io(format!("I/O error: {}", err))
    }
}

/// Convert serde_json::Error to ShipLabelError
impl From<serde_json::Error> for ShipLabelError {
    fn from(err: serde_json::Error) -> Self {
        ShipLabelError::Data(format!("JSON parsing error: {}", err))
    }
}

/// Convert qrcode::types::QrError to ShipLabelError
impl From<types::QrError> for ShipLabelError {
    fn from(err: types::QrError) -> Self {
        ShipLabelError::Data(format!("QR code generation error: {}", err))
    }
}

/// Convert image::ImageError to ShipLabelError
impl From<image::ImageError> for ShipLabelError {
    fn from(err: image::ImageError) -> Self {
        ShipLabelError::Data(format!("Image processing error: {}", err))
    }
}

/// Convert Box<dyn std::error::Error> to ShipLabelError
impl From<Box<dyn std::error::Error>> for ShipLabelError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ShipLabelError::Generic(format!("External error: {}", err))
    }
}