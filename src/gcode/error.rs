//! Error types for G-code handling.

use thiserror::Error;

/// Result type alias for G-code operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur when parsing G-code.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing float value.
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] core::num::ParseFloatError),

    /// Error parsing integer value.
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),

    /// Invalid character encountered.
    #[error("Invalid character: {0}")]
    InvalidCharacter(char),

    /// Unexpected end of input.
    #[error("Unexpected end of input")]
    UnexpectedEnd,

    /// Unclosed comment.
    #[error("Unclosed comment")]
    UnclosedComment,

    /// UTF-8 error.
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}
