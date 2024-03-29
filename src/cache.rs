//! # ResponseCache Module
//!
//! It holds a single lookup response and the time it was created. The cached response can be saved and loaded from disk.
//! It also provides a method to delete the cache from disk.
//!
//! ## Example
//! ```rust
//! use std::error::Error;
//! use public_ip_address::{cache::ResponseCache, response::LookupResponse};
//! use public_ip_address::lookup::LookupProvider;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let response_cache = ResponseCache::new(LookupResponse::new(
//!         "1.1.1.1".to_string(),
//!         LookupProvider::IpBase,
//!     ));
//!     response_cache.save()?;
//!     let cached = ResponseCache::load()?;
//!     println!("{:?}", cached);
//!     cached.delete()?;
//!     Ok(())
//! }
//! ```

use crate::{error::CacheError, LookupResponse};
use base64::prelude::*;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, fs::File, io::prelude::*, time::SystemTime};

/// Result type wrapper for the cache
pub type Result<T> = std::result::Result<T, CacheError>;

/// Holds the response and the time it was saved
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseCache {
    pub response: LookupResponse,
    pub response_time: SystemTime,
}

impl ResponseCache {
    /// Creates a new `ResponseCache` instance.
    ///
    /// # Arguments
    ///
    /// * `response` - A `LookupResponse` to be cached.
    pub fn new(response: LookupResponse) -> ResponseCache {
        ResponseCache {
            response,
            response_time: SystemTime::now(),
        }
    }

    /// Saves the `ResponseCache` instance to disk.
    ///
    /// This function serializes the `ResponseCache` instance to a JSON string, encodes it using Base64, and then writes it to a file on disk.
    /// The file is located at the path returned by the `get_cache_path` function.
    pub fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string(self)?;
        let encoded = BASE64_STANDARD.encode(serialized);
        let mut file = File::create(get_cache_path())?;
        file.write_all(encoded.as_bytes())?;
        Ok(())
    }

    /// Loads the `ResponseCache` instance from disk.
    pub fn load() -> Result<ResponseCache> {
        let mut file = File::open(get_cache_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let decoded = String::from_utf8(BASE64_STANDARD.decode(contents)?)?;
        let deserialized: ResponseCache = serde_json::from_str(&decoded)?;
        Ok(deserialized)
    }

    /// Deletes the `ResponseCache` instance from disk.
    pub fn delete(self) -> Result<()> {
        fs::remove_file(get_cache_path())?;
        Ok(())
    }
}

/// Returns the path to the cache file.
///
/// This function attempts to get the system's cache directory using the `BaseDirs` struct.
/// If the cache directory doesn't exist, it attempts to create it.
/// If it can't create the cache directory, it falls back to the home directory.
/// If it can't get the home directory, it falls back to the current directory.
/// The cache file is named ".lookupcache".
pub fn get_cache_path() -> String {
    if let Some(base_dirs) = BaseDirs::new() {
        let mut dir = base_dirs.cache_dir();
        // Create cache directory if it doesn't exist
        if !dir.exists() && fs::create_dir_all(dir).is_err() {
            // If we can't create the cache directory, fallback to home directory
            dir = base_dirs.home_dir();
        }
        if let Some(path) = dir.join(".lookupcache").to_str() {
            return path.to_string();
        }
    };
    // If we can't get the cache directory, fallback to current directory
    ".lookupcache".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::LookupProvider;

    #[test]
    fn test_cache() {
        let response = LookupResponse::new(
            "1.1.1.1".to_string(),
            LookupProvider::Mock("1.1.1.1".to_string()),
        );
        let cache = ResponseCache::new(response);
        cache.save().unwrap();
        let cached = ResponseCache::load().unwrap();
        assert_eq!(cached.response.ip, "1.1.1.1", "IP address not matching");
        cache.delete().unwrap();
    }
}
