//! Lookup error types
use thiserror::Error;

/// Result type for the lookup crate
pub type Result<T> = std::result::Result<T, LookupError>;

/// Error type for the lookup process
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LookupError {
    #[error("Reqwuest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Too many API requests")]
    TooManyRequests(String),
    #[error("Request status")]
    RequestStatus(String),
    #[error("Serde error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Lookup error")]
    GenericError(String),
}
