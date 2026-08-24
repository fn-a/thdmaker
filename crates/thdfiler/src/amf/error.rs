//! Error types for AMF file handling.

use thiserror::Error;

/// Result type alias for AMF operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading or writing AMF files.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP archive error.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// XML parsing error.
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// XML encoding error.
    #[error("XML encoding error: {0}")]
    XmlEncoding(#[from] quick_xml::encoding::EncodingError),

    /// XML parsing error.
    #[error("XML escape error: {0}")]
    XmlEscape(#[from] quick_xml::escape::EscapeError),

    /// XML deserialization error.
    #[error("XML deserialization error: {0}")]
    XmlDeserialize(#[from] quick_xml::DeError),

    /// Error parsing float value.
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    /// UTF-8 error.
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Error converting from UTF-8.
    #[error("From UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// Unexpected error occurred.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}