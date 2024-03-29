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
    perform_cached_lookup_with_list(
        vec![
            LookupProvider::IpInfo,
            LookupProvider::IpWhoIs,
            LookupProvider::MyIp,
            LookupProvider::FreeIpApi,
        ],
        None,
    )
}

/// Performs lookup with a specific service provider.
pub fn perform_lookup_with(provider: LookupProvider) -> Result<LookupResponse> {
    Ok(LookupService::new(provider).make_request()?)
}
/// Performs lookup with a list of specific service providers.
pub fn perform_lookup_with_list(providers: Vec<LookupProvider>) -> Result<LookupResponse> {
    let mut error = None;
    if providers.is_empty() {
        return Err(Error::LookupErrorString("No providers given".to_string()));
    }

    for provider in providers {
        let response = LookupService::new(provider).make_request();
        if let Ok(response) = response {
            return Ok(response);
        }
        error = Some(response.unwrap_err());
    }

    // if we reach here no responses were found
    Err(Error::LookupErrorString(format!(
        "No responses from providers: {:?}",
        error
    )))
}

/// Performs lookup with a specific service provider.
///
/// The result is cached locally and if subsequen requests are made, the cached result is returned
/// as long as the previous request was made within `cache_time` seconds. If `cache_time` is `None`
/// then the `DEFAULT_CACHE_TIME` is used.
pub fn perform_cached_lookup_with(
    provider: LookupProvider,
    cache_time: Option<u64>,
) -> Result<LookupResponse> {
    let cached = ResponseCache::load();
    if let Ok(cache) = cached {
        let difference = SystemTime::now().duration_since(cache.response_time)?;
        // check if cache expired
        if difference <= Duration::from_secs(cache_time.unwrap_or(DEFAULT_CACHE_TIME)) {
            return Ok(cache.response);
        }
    }

    let service = LookupService::new(provider);
    // no cache or it's too old, make a new request.
    match service.make_request() {
        Ok(result) => {
            let cache = ResponseCache::new(result);
            cache.save()?;
            Ok(cache.response)
        }
        Err(e) => Err(Error::LookupError(e)),
    }
}

/// Performs lookup with a list of specific service providers.
///
/// The result is cached locally and if subsequen requests are made, the cached result is returned
/// as long as the previous request was made within `cache_time` seconds. If `cache_time` is `None`
/// then the `DEFAULT_CACHE_TIME` is used.
pub fn perform_cached_lookup_with_list(
    providers: Vec<LookupProvider>,
    cache_time: Option<u64>,
) -> Result<LookupResponse> {
    let cached = ResponseCache::load();
    if let Ok(cache) = cached {
        let difference = SystemTime::now().duration_since(cache.response_time)?;
        // check if cache expired
        if difference <= Duration::from_secs(cache_time.unwrap_or(DEFAULT_CACHE_TIME)) {
            return Ok(cache.response);
        }
    }

    // no cache or it's too old, make a new request.
    match perform_lookup_with_list(providers) {
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
        let response = perform_lookup_with(LookupProvider::Mock("1.1.1.1".to_string()));
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_perform_lookup_cached() {
        let response =
            perform_cached_lookup_with(LookupProvider::Mock("1.1.1.1".to_string()), None);
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_perform_lookup_list() {
        let response = perform_lookup_with_list(vec![LookupProvider::Mock("1.1.1.1".to_string())]);
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_perform_lookup_list_cached() {
        let response = perform_cached_lookup_with_list(
            vec![LookupProvider::Mock("1.1.1.1".to_string())],
            None,
        );
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }
}
