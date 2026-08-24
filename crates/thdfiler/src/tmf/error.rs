//! Error types for 3MF file handling.

use thiserror::Error;

/// Result type alias for 3MF operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading or writing 3MF files.
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

    /// Unexpected end of file.
    #[error("Unexpected EOF in: {0}")]
    UnexpectedEofIn(String),

    /// Missing required part.
    #[error("Missing required part: {0}")]
    MissingRequired(String),

    /// Invalid 3MF structure.
    #[error("Invalid 3MF structure: {0}")]
    InvalidStructure(String),

    /// Invalid resource reference.
    #[error("Invalid resource reference: {0}")]
    InvalidReference(String),

    /// Invalid attribute value.
    #[error("Invalid attribute value for '{name}': {message}")]
    InvalidAttribute { name: String, message: String },

    /// Invalid color format.
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    /// Invalid matrix format.
    #[error("Invalid matrix format: {0}")]
    InvalidMatrix(String),

    /// Invalid mesh (not manifold, etc.).
    #[error("Invalid mesh: {0}")]
    InvalidMesh(String),

    /// Resource ID already exists.
    #[error("Duplicate resource ID: {0}")]
    DuplicateResourceId(u32),

    /// UTF-8 encoding error.
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Error converting from UTF-8.
    #[error("From UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// Integer parsing error.
    #[error("Integer parsing error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    /// Float parsing error.
    #[error("Float parsing error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    /// Pattern error.
    #[error("Pattern error: {0}")]
    PatternError(#[from] glob::PatternError),

    /// UUID error.
    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),

    /// Base64 decoding error.
    #[error("Base64 decoding error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
}