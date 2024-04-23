//! # ❌ Crate errors

use crate::lookup::error::LookupError;
use thiserror::Error;

/// Result type wrapper for the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the crate
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Cache error when reading or writing to the cache
    #[error("Cache error")]
    CacheError(#[from] CacheError),
    /// Lookup error when fetching information about an IP address
    #[error("Lookup error")]
    LookupError(#[from] LookupError),
    /// System time error, usually when converting from a timestamp
    #[error("Time error")]
    TimeError(#[from] std::time::SystemTimeError),
}

/// Error type for the cache module
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CacheError {
    /// Serde error when serializing or deserializing data
    #[error("Serde error")]
    SerdeError(#[from] serde_json::Error),
    /// IO error when reading or writing to the cache
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    /// Utf8 error when converting from bytes to string
    #[error("Utf8 error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    /// Encryption error when encrypting or decrypting data
    #[error("Encryption error")]
    EncryptionError(String),
}
