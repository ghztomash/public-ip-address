//! # 🔎 Public IP Address Lookup and Geolocation Information
//!
//! `public-ip-address` is a simple, easy-to-use Rust library for performing public IP lookups from various services.
//! It provides a unified interface to fetch public IP address and geolocation information from multiple providers.
//! The library also includes caching functionality to improve performance for repeated lookups and minimaze rate-limiting.
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
//!
//! ## Features
//! - Unified interface for multiple IP lookup providers
//! - Caching of lookup results to improve performance
//! - Customizable cache expiration time
//!
//! For more details, please refer to the API documentation.

use cache::ResponseCache;
use error::{Error, Result};
use lookup::{error::LookupError, LookupProvider, LookupService};
use response::LookupResponse;
use std::time::{Duration, SystemTime};

pub mod cache;
pub mod error;
pub mod lookup;
pub mod response;

/// Performs a lookup using a predefined list of `LookupProvider`s and caches the result.
///
/// This function performs a lookup using a predefined list of `LookupProvider`s. The list includes
/// `IpInfo`, `IpWhoIs`, `MyIp`, and `FreeIpApi`. The result of the lookup is cached locally for 2 seconds.
/// If a subsequent request is made within 2 seconds, the cached result is returned.
///
/// # Example
///
/// ```
/// match public_ip_address::perform_lookup() {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// ```
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or an `Error` if the lookup or caching failed.
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

/// Performs a lookup using a list of providers until a successful response is received.
///
/// This function iterates over the provided list of `LookupProvider`s, making a request with each one
/// until a successful `LookupResponse` is received. If a provider fails to return a successful response,
/// the error is stored and the next provider is tried.
///
/// If all providers fail to return a successful response, a `LookupError` is returned containing a list
/// of all the errors received.
///
/// # Arguments
///
/// * `providers` - A vector of `LookupProvider`s to use for the lookup.
///
/// # Example
///
/// ```rust
/// use public_ip_address::lookup::LookupProvider;
///
/// let providers = vec![
///     // List of providers to use for the lookup
///     // LookupProvider::IpWhoIs,
/// ];
///
/// match public_ip_address::perform_lookup_with(providers) {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// ```
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or a `LookupError` containing a list of all errors received.
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

/// Performs a lookup with a list of specific service providers and caches the result.
///
/// This function performs a lookup using the provided list of `LookupProvider`s. The result of the lookup
/// is cached locally.
/// If subsequent requests are made, the cached result is returned as long as the previous
/// request was made within `cache_expire_time` seconds.
///
/// If `cache_expire_time` is `None`, then the cache never expires.
///
/// If `cache_expire_time` is `0`, then the cache is forced to expire immediately after the request.
///
/// # Arguments
///
/// * `providers` - A vector of `LookupProvider`s to use for the lookup.
/// * `cache_expire_time` - An `Option` containing the number of seconds before the cache expires. If `None`,
///   the cache never expires. If `0`, the cache expires immediately after the request.
///
/// # Example
///
/// ```rust
/// use public_ip_address::lookup::LookupProvider;
///
/// let providers = vec![
///     // List of providers to use for the lookup
///     // LookupProvider::IpWhoIs,
/// ];
/// let expire_time = Some(60); // Cache expires after 60 seconds
///
/// match public_ip_address::perform_cached_lookup_with(providers, expire_time) {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// ```
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or an `Error` if the lookup or caching failed.
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
    use std::net::IpAddr;

    #[test]
    fn test_perform_lookup() {
        let response = perform_lookup_with(vec![LookupProvider::Mock("1.1.1.1".to_string())]);
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }

    #[test]
    fn test_perform_lookup_cached() {
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("1.1.1.1".to_string())], Some(0));
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }

    #[test]
    fn test_perform_lookup_cached_expired() {
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("1.1.1.1".to_string())], Some(0));
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("2.2.2.2".to_string())], Some(1));
        assert!(response.is_ok());
        // the old cache should be returned
        assert_eq!(
            response.unwrap().ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "The old cache should be returned"
        );
        let response =
            perform_cached_lookup_with(vec![LookupProvider::Mock("2.2.2.2".to_string())], Some(0));
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            "2.2.2.2".parse::<IpAddr>().unwrap(),
            "Cached value should expire"
        );
    }
}
