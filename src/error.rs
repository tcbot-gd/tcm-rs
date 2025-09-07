//! Error types for the TCM library.
//!
//! This module provides comprehensive error handling for all TCM operations.

use thiserror::Error;

/// Main error type for all TCM operations.
#[derive(Error, Debug)]
pub enum TcmError {
    /// I/O errors during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid TCM file format or corrupted data
    #[error("Invalid format: {message}")]
    InvalidFormat { message: String },

    /// Unsupported format version
    #[error("Unsupported format version: {version}")]
    UnsupportedVersion { version: u8 },

    /// Invalid input data
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Invalid button value
    #[error("Invalid button value: {value} (expected 1-3)")]
    InvalidButton { value: u8 },

    /// Invalid restart type
    #[error("Invalid restart type: {value} (expected 0-2)")]
    InvalidRestartType { value: u8 },

    /// Invalid metadata
    #[error("Invalid metadata: {message}")]
    InvalidMetadata { message: String },

    /// File header validation failed
    #[error("Invalid file header - not a valid TCM file")]
    InvalidHeader,

    /// Unexpected end of file
    #[error("Unexpected end of file while reading {context}")]
    UnexpectedEof { context: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Deserialization error
    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },
}

/// Result type alias for TCM operations.
pub type TcmResult<T> = Result<T, TcmError>;

impl TcmError {
    /// Creates an invalid format error with a message.
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }

    /// Creates an invalid input error with a message.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    /// Creates an invalid metadata error with a message.
    pub fn invalid_metadata(message: impl Into<String>) -> Self {
        Self::InvalidMetadata {
            message: message.into(),
        }
    }

    /// Creates an unexpected EOF error with context.
    pub fn unexpected_eof(context: impl Into<String>) -> Self {
        Self::UnexpectedEof {
            context: context.into(),
        }
    }

    /// Creates a serialization error with a message.
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Creates a deserialization error with a message.
    pub fn deserialization_error(message: impl Into<String>) -> Self {
        Self::DeserializationError {
            message: message.into(),
        }
    }
}
