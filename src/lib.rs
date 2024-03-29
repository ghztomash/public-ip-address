//! # Public IP address
//!
//! A simple library for performing public IP lookups from various services.
//!
//! ## Usage
//! ```toml
//! [dependencies]
//! public-ip-address = { version = "0.1" }
//! ```
//! ## Example
//! ```rust
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let result = public_ip_address::perform_lookup()?;
//!     println!("{}", result);
//!     Ok(())
//! }
//! ```

use cache::ResponseCache;
use error::{Error, Result};
use lookup::{error::LookupError, LookupProvider, LookupService};
use response::LookupResponse;
use std::time::{Duration, SystemTime};

pub mod cache;
pub mod error;
pub mod lookup;
pub mod response;

/// Perform a simple lookup.
/// This calls `perform_cached_lookup_with_list()` with default values.
pub fn perform_lookup() -> Result<LookupResponse> {
    perform_cached_lookup_with(
        vec![
            LookupProvider::IpInfo,
            LookupProvider::IpWhoIs,
            LookupProvider::MyIp,
            LookupProvider::FreeIpApi,
        ],
        Some(2),
    )
}

/// Performs lookup with a list of specific service providers.
///
/// Providers are called in the order they are given.
/// The first provider that returns a successful response is returned.
pub fn perform_lookup_with(providers: Vec<LookupProvider>) -> Result<LookupResponse> {
    let mut errors = Vec::new();
    if providers.is_empty() {
        return Err(Error::LookupError(LookupError::GenericError(
            "No providers given".to_string(),
        )));
    }

    for provider in providers {
        let response = LookupService::new(provider).make_request();
        if let Ok(response) = response {
            return Ok(response);
        }
        errors.push(response.unwrap_err());
    }

    // if we reach here no responses were found
    Err(Error::LookupError(LookupError::GenericError(format!(
        "No responses from providers: {:?}",
        errors
    ))))
}

/// Performs lookup with a list of specific service providers.
///
/// The result is cached locally and if subsequent requests are made, the cached result is returned
/// as long as the previous request was made within `cache_expire_time` seconds.
/// If `cache_time` is `None` then the cache never expires.
/// If `cache_expire_time` is `0` then the cache is forced to expire.
pub fn perform_cached_lookup_with(
    providers: Vec<LookupProvider>,
    cache_expire_time: Option<u64>,
) -> Result<LookupResponse> {
    let cached = ResponseCache::load();
    if let Ok(cache) = cached {
        let difference = SystemTime::now().duration_since(cache.response_time)?;
        // check if cache expired
        if let Some(cache_expire_time) = cache_expire_time {
            if difference <= Duration::from_secs(cache_expire_time) {
                return Ok(cache.response);
            }
        } else {
            // cache never expires
            return Ok(cache.response);
        }
    }

    // no cache or it's too old, make a new request.
    match perform_lookup_with(providers) {
        Ok(result) => {
            let cache = ResponseCache::new(result);
            cache.save()?;
            Ok(cache.response)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_lookup() {
        let response = perform_lookup_with(vec![LookupProvider::Mock("1.1.1.1".to_string())]);
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_perform_lookup_cached() {
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("1.1.1.1".to_string())], Some(0));
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_perform_lookup_cached_expired() {
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("1.1.1.1".to_string())], Some(0));
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("2.2.2.2".to_string())], Some(1));
        assert!(response.is_ok());
        // the old cache should be returned
        assert_eq!(
            response.unwrap().ip,
            "1.1.1.1",
            "The old cache should be returned"
        );
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("2.2.2.2".to_string())], Some(0));
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            "2.2.2.2",
            "Cached value should expire"
        );
    }
}
