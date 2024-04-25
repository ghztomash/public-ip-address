//! Lookup error types
use thiserror::Error;

/// Result type for the lookup crate
pub type Result<T> = std::result::Result<T, LookupError>;

/// Error type for the lookup process
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LookupError {
    /// API request error
    #[error("Reqwuest error")]
    ReqwestError(#[from] reqwest::Error),
    /// Too many requests
    #[error("Too many API requests")]
    TooManyRequests(String),
    /// Other HTTP code
    #[error("Request status")]
    RequestStatus(String),
    /// Serde error
    #[error("Serde error")]
    SerdeError(#[from] serde_json::Error),
    /// Generic error
    #[error("Lookup error")]
    GenericError(String),
    /// Target address not supported by this provider
    #[error("Target lookup not supported")]
    TargetNotSupported,
}
