//! Error types for STL file handling.

use thiserror::Error;

/// Result type alias for STL operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading or writing STL files.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing float value.
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    /// Invalid ASCII segment.
    #[error("Invalid segment: expected {expected}, received {received}")]
    InvalidSegment{ expected: String, received: String },

    /// No mesh data to write.
    #[error("No mesh data to write")]
    EmptyMeshError,

    /// Non-finite value (NaN or infinity) in mesh data.
    #[error("Non-finite value (NaN or inf) in mesh data")]
    NonFiniteValue,

    /// Unexpected error occurred.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}