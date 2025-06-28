//! # 🗄️ Response cache Module
//!
//! This module provides a `ResponseCache` struct that holds the current IP address lookup response and the time it was created, and when it should expire.
//!
//! The `ResponseCache` can be saved to disk, loaded from disk, and deleted from disk. It also provides methods to clear the cache,
//! update the cache with a new response, check if the cache has expired, and retrieve the IP address or the entire response from the cache.
//!
//! The cache is stored in a JSON format by default in the system cache directory. And a custom file name can be provided.
//!
//! If the `encryption` feature is enabled, the cache is encrypted using AEAD.
//!
//! ## Example
//! ```rust
//! use std::error::Error;
//! use public_ip_address::{cache::ResponseCache, response::LookupResponse};
//! use public_ip_address::lookup::LookupProvider;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let response = LookupResponse::new(
//!             "1.1.1.1".parse::<std::net::IpAddr>()?,
//!             LookupProvider::IpBase,
//!     );
//!     let mut response_cache = ResponseCache::new(None);
//!     response_cache.update_current(&response, None);
//!     response_cache.save()?;
//!     let cached = ResponseCache::load(None)?;
//!     println!("{:?}", cached);
//!     cached.delete()?;
//!     Ok(())
//! }
//! ```

use crate::{error::CacheError, LookupResponse};
use etcetera::{choose_base_strategy, BaseStrategy};
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    fs::File,
    io::prelude::*,
    net::IpAddr,
    time::{Duration, SystemTime},
};

#[cfg(feature = "encryption")]
use cocoon::Cocoon;

/// Result type wrapper for the cache
pub type Result<T> = std::result::Result<T, CacheError>;

/// Represents an entry of the cached response
///
/// It contains the `LookupResponse`, the time when the response was cached, and the time-to-live (TTL) of the cache.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[non_exhaustive]
pub struct ResponseRecord {
    /// Cached response
    pub response: LookupResponse,
    response_time: SystemTime,
    ttl: Option<u64>,
}

impl ResponseRecord {
    /// Creates a new `ResponseRecord` instance.
    ///
    /// # Arguments
    ///
    /// * `response` - A `LookupResponse` to be cached.
    /// * `ttl` - An optional `u64` value representing after how many seconds the cached value expires.
    ///   None means the cache never expires.
    pub fn new(response: LookupResponse, ttl: Option<u64>) -> ResponseRecord {
        ResponseRecord {
            response,
            response_time: SystemTime::now(),
            ttl,
        }
    }

    /// Determines if the cached response has expired.
    ///
    /// If the TTL is not set, the function assumes that the cache never expires and returns false.
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let difference = SystemTime::now()
                .duration_since(self.response_time)
                .unwrap_or_default();
            difference >= Duration::from_secs(ttl)
        } else {
            // No TTL, cache never expires
            false
        }
    }

    /// Returns the IP address of the cached response.
    pub fn ip(&self) -> std::net::IpAddr {
        self.response.ip
    }
}

/// Holds the current IP address lookup response
///
/// The cache can be saved to disk, loaded from disk, and deleted from disk. It also provides methods to clear the cache,
/// update the cache with a new response, check if the cache has expired, and retrieve the IP address or the entire response from the cache.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ResponseCache {
    /// The current IP address lookup response
    pub current_address: Option<ResponseRecord>,
    /// A tree of arbitrary IP address responses
    pub lookup_address: BTreeMap<IpAddr, ResponseRecord>,
    /// The cache file name
    file_name: Option<String>,
}

impl ResponseCache {
    /// Creates a new `ResponseCache` instance.
    ///
    /// The `ResponseRecord` is stored as the `current_address` in the `ResponseCache`.
    ///
    /// # Arguments
    ///
    /// * `file_name` - An `Option<String>` representing the name of the file where the cache will be stored. If `None`, no file will be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use public_ip_address::cache::ResponseCache;
    /// # use public_ip_address::lookup::LookupProvider;
    /// # use public_ip_address::response::LookupResponse;
    /// let response = LookupResponse::new(
    ///             "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
    ///             LookupProvider::IpBase);
    /// let mut cache = ResponseCache::new(None);
    /// cache.update_current(&response, None);
    /// ```
    ///
    /// ```
    /// # use public_ip_address::cache::ResponseCache;
    /// let cache = ResponseCache::new(Some("cache.txt".to_string()));
    /// ```
    ///
    pub fn new(file_name: Option<String>) -> ResponseCache {
        trace!("Creating new cache structure");
        ResponseCache {
            current_address: None,
            lookup_address: BTreeMap::new(),
            file_name,
        }
    }

    /// Clears the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// # use public_ip_address::cache::ResponseCache;
    /// let mut cache = ResponseCache::default();
    /// cache.clear();
    /// assert!(cache.current_response().is_none());
    /// ```
    pub fn clear(&mut self) {
        trace!("Clearing cache");
        self.current_address = None;
        self.lookup_address.clear();
    }

    /// Updates the cache entry for the current host with a new response.
    ///
    /// # Arguments
    ///
    /// * `response` - A `LookupResponse` instance representing the new address to be cached.
    /// * `ttl` - An `Option<u64>` representing the time-to-live (TTL) in seconds for the new cached response. If `None`, the cache never expires.
    ///
    pub fn update_current(&mut self, response: &LookupResponse, ttl: Option<u64>) {
        self.current_address = Some(ResponseRecord::new(response.to_owned(), ttl));
    }

    /// Checks if the `current_address` cache entry has expired.
    pub fn current_is_expired(&self) -> bool {
        match self.current_address {
            Some(ref current) => current.is_expired(),
            None => true,
        }
    }

    /// Returns the IP address of the current host cache entry.
    pub fn current_ip(&self) -> Option<std::net::IpAddr> {
        self.current_address.as_ref().map(|current| current.ip())
    }

    /// Returns the `current_address` cache entry.
    pub fn current_response(&self) -> Option<LookupResponse> {
        self.current_address
            .as_ref()
            .map(|current| current.response.to_owned())
    }

    /// Updates the lookup cache with a new response.
    pub fn update_target(&mut self, ip: IpAddr, response: &LookupResponse, ttl: Option<u64>) {
        self.lookup_address
            .insert(ip, ResponseRecord::new(response.to_owned(), ttl));
    }

    /// Updates the lookup cache with a new responses.
    pub fn update_targets(&mut self, responses: &[(&IpAddr, LookupResponse)], ttl: Option<u64>) {
        for (ip, response) in responses {
            self.lookup_address.insert(
                *ip.to_owned(),
                ResponseRecord::new(response.to_owned(), ttl),
            );
        }
    }

    /// Checks if the lookup cache entry for the given IP address has expired.
    pub fn target_is_expired(&self, ip: &IpAddr) -> bool {
        match self.lookup_address.get(ip) {
            Some(lookup) => lookup.is_expired(),
            None => true,
        }
    }

    /// Checks if the lookup cache entry for at least one of IP addresses has expired.
    pub fn targets_contain_expired(&self, ips: &[IpAddr]) -> bool {
        for ip in ips {
            if self.target_is_expired(ip) {
                return true;
            }
        }
        false
    }

    /// Returns lookup cached entry for the given IP address.
    pub fn target_response(&self, ip: &IpAddr) -> Option<LookupResponse> {
        self.lookup_address
            .get(ip)
            .map(|lookup| lookup.response.to_owned())
    }

    /// Writes the `ResponseCache` instance to a file on disk.
    ///
    /// This method serializes the `ResponseCache` instance into a JSON string, encrypts the data if the "encryption" feature is enabled,
    /// and then writes the encrypted (or plain text) data to a file. The file is located at the path specified by the `file_name` field of the `ResponseCache` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use public_ip_address::cache::ResponseCache;
    /// let cache = ResponseCache::new(Some("cache.txt".to_string()));
    /// _ = cache.save();
    /// ```
    pub fn save(&self) -> Result<()> {
        debug!("Saving cache to {}", get_cache_path(&self.file_name));
        let data = serde_json::to_string(self)?.into_bytes();

        #[cfg(feature = "encryption")]
        let data = encrypt(data)?;

        let mut file = File::create(get_cache_path(&self.file_name))?;
        file.write_all(&data)?;
        Ok(())
    }

    /// Loads the `ResponseCache` instance from a file on disk.
    ///
    /// This method reads the file specified by `file_name`, decrypts the data if the "encryption" feature is enabled,
    /// and then deserializes the data into a `ResponseCache` instance.
    ///
    /// # Arguments
    ///
    /// * `file_name` - An `Option<String>` representing the name of the file from which the cache will be loaded.
    ///   If `None`, the default file name `lookup.cache` will be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use public_ip_address::cache::ResponseCache;
    /// let cache = ResponseCache::load(Some("cache.txt".to_string()));
    /// ```
    pub fn load(file_name: Option<String>) -> Result<ResponseCache> {
        debug!("Loading cache from {}", get_cache_path(&file_name));
        let mut file = File::open(get_cache_path(&file_name))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        #[cfg(feature = "encryption")]
        let data = decrypt(data)?;

        let decoded = String::from_utf8(data).unwrap_or_default();
        let deserialized: ResponseCache = serde_json::from_str(&decoded)?;
        Ok(deserialized)
    }

    /// Deletes the `ResponseCache` instance from disk.
    pub fn delete(self) -> Result<()> {
        trace!("Deleting cache file {}", get_cache_path(&self.file_name));
        fs::remove_file(get_cache_path(&self.file_name))?;
        Ok(())
    }
}

/// Determines the path for the cache file.
///
/// This function uses a series of fallbacks to find a suitable directory for the cache file:
/// 1. It first tries to use the system's cache directory, as determined by the `BaseStrategy`.
/// 2. If the cache directory doesn't exist, it tries to create it.
/// 3. If it can't create the cache directory, it falls back to the system's data directory.
/// 4. If it can't use the data directory, it falls back to the user's home directory.
/// 5. If it can't use the home directory, it falls back to the current directory.
///
/// The cache file is named "lookup.cache" by default. However, this can be overridden by providing a different name as a parameter.
///
/// # Arguments
///
/// * `file_name` - An `Option<String>` representing the desired name of the cache file. If `None`, the default name "lookup.cache" is used.
///
/// # Returns
///
/// * `String` - The path to the cache file.
///
/// # Examples
///
/// ```
/// # use public_ip_address::cache::get_cache_path;
/// let cache_path = get_cache_path(&Some("my_cache.txt".to_string()));
/// ```
pub fn get_cache_path(file_name: &Option<String>) -> String {
    let file_name = if let Some(file_name) = file_name {
        file_name
    } else {
        "lookup.cache"
    };

    if let Ok(base_dirs) = choose_base_strategy() {
        let mut dir = base_dirs.cache_dir();
        // Create cache directory if it doesn't exist
        if !dir.exists() && fs::create_dir_all(&dir).is_err() {
            // If we can't create the cache directory, fallback to data directory
            dir = base_dirs.data_dir();
            if !dir.exists() && fs::create_dir_all(&dir).is_err() {
                // If we can't create the data directory, fallback to home directory
                dir = base_dirs.home_dir().to_path_buf();
            }
        }
        if let Some(path) = dir.join(file_name).to_str() {
            return path.to_string();
        }
    };
    // As last resort, fallback to current directory
    file_name.to_string()
}

/// Decrypts the given data using AEAD.
///
/// In debug mode, it uses a weaker key derivation function for faster speed.
///
/// # Arguments
///
/// * `data` - The data to be decrypted, as a vector of bytes.
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The decrypted data, as a vector of bytes.
/// * `Err(CacheError::EncryptionError)` - If there was an error during decryption.
#[cfg(feature = "encryption")]
fn decrypt(data: Vec<u8>) -> Result<Vec<u8>> {
    trace!("Decrypting data");
    let password = mid::get(env!("CARGO_PKG_NAME")).unwrap_or("lookup".to_string());
    let cocoon = if cfg!(debug_assertions) {
        Cocoon::new(password.as_bytes()).with_weak_kdf()
    } else {
        Cocoon::new(password.as_bytes())
    };
    match cocoon.unwrap(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(CacheError::EncryptionError(format!(
            "Error decrypting: {:?}",
            e
        ))),
    }
}

/// Encrypts the given data using AEAD.
///
/// In debug mode, it uses a weaker key derivation function for faster speed.
///
/// # Arguments
///
/// * `data` - The data to be encrypted, as a vector of bytes.
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The encrypted data, as a vector of bytes.
/// * `Err(CacheError::EncryptionError)` - If there was an error during encryption.
#[cfg(feature = "encryption")]
fn encrypt(data: Vec<u8>) -> Result<Vec<u8>> {
    trace!("Encrypting data");
    let password = mid::get(env!("CARGO_PKG_NAME")).unwrap_or("lookup".to_string());
    let mut cocoon = if cfg!(debug_assertions) {
        Cocoon::new(password.as_bytes()).with_weak_kdf()
    } else {
        Cocoon::new(password.as_bytes())
    };
    match cocoon.wrap(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(CacheError::EncryptionError(format!(
            "Error encrypting: {:?}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::LookupProvider;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_cache_file() {
        let response = LookupResponse::new(
            "1.1.1.1".parse().unwrap(),
            LookupProvider::Mock("1.1.1.1".to_string()),
        );
        println!("{}", get_cache_path(&None));
        let mut cache = ResponseCache::new(None);
        cache.update_current(&response, None);
        cache.save().unwrap();
        let cached = ResponseCache::load(None).unwrap();
        assert_eq!(
            cached.current_ip().unwrap(),
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
        cache.delete().unwrap();
    }

    #[test]
    fn test_expired() {
        let response = LookupResponse::new(
            "1.1.1.1".parse().unwrap(),
            LookupProvider::Mock("1.1.1.1".to_string()),
        );
        let mut cache = ResponseCache::default();
        assert!(cache.current_is_expired(), "Empty cache should be expired");
        cache.update_current(&response, None);
        assert_eq!(
            cache.current_ip().unwrap(),
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
        assert!(
            !cache.current_is_expired(),
            "Cache with no TTL should not be expired"
        );
        cache.update_current(&response, Some(1));
        assert!(
            !cache.current_is_expired(),
            "Fresh cache should not be expired {:#?}",
            cache
        );
        // Wait for cache to expire
        std::thread::sleep(Duration::from_secs(1));
        assert!(
            cache.current_is_expired(),
            "Expired cache should be expired"
        );
    }

    #[test]
    fn test_cache_tree() {
        let addresses = [
            "1.1.1.1".parse().unwrap(),
            "2.1.1.1".parse().unwrap(),
            "3.1.1.1".parse().unwrap(),
        ];
        let mut cache = ResponseCache::default();
        for address in &addresses {
            let response = LookupResponse::new(*address, LookupProvider::Ipify);
            cache.update_target(*address, &response, None);
        }

        for address in &addresses {
            assert_eq!(
                cache.target_response(address).unwrap().ip,
                *address,
                "IP address not matching: {:#?}",
                cache
            );
        }
    }

    #[test]
    fn test_cache_clear() {
        let response = LookupResponse::new(
            "1.1.1.1".parse().unwrap(),
            LookupProvider::Mock("1.1.1.1".to_string()),
        );
        let mut cache = ResponseCache::new(None);
        cache.update_current(&response, None);
        let response = LookupResponse::new("2.2.2.2".parse().unwrap(), LookupProvider::Ipify);
        cache.update_target(response.ip, &response, None);
        cache.clear();
        let cache = ResponseCache::default();
        assert_eq!(
            cache,
            ResponseCache::default(),
            "Cache not cleared properly: {:#?}",
            cache
        );
    }

    #[test]
    #[cfg(feature = "encryption")]
    fn test_encrypt_decrypt() {
        let data = b"hello world".to_vec();
        let encrypted = encrypt(data.clone()).unwrap();
        let decrypted = decrypt(encrypted).unwrap();
        assert_eq!(data, decrypted);
    }
}
