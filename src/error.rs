use thiserror::Error;

/// Result type for the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the crate
#[derive(Error, Debug)]
pub enum Error {
    #[error("Cache error")]
    CacheError(#[from] CacheError),
    #[error("Lookup error")]
    LookupError(#[from] LookupError),
    #[error("Lookup error")]
    LookupErrorString(String),
    #[error("Time error")]
    TimeError(#[from] std::time::SystemTimeError),
}

/// Error type for the lookup process
#[derive(Error, Debug)]
pub enum LookupError {
    #[error("Reqwuest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Too many API requests")]
    TooManyRequests(String),
    #[error("Request status")]
    RequestStatus(String),
    #[error("Serde error")]
    SerdeError(#[from] serde_json::Error),
}

/// Error type for the cache
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Serde error")]
    SerdeError(#[from] serde_json::Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Base64 error")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Utf8 error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
