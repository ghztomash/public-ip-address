//! # 🔎 Public IP Address Lookup and Geolocation Information
//!
//! `public-ip-address` is a simple, easy-to-use Rust library for performing public IP lookups from various services.
//! It provides a unified interface to fetch public IP address and geolocation information from multiple providers.
//! The library also includes caching functionality to improve performance for repeated lookups and minimaze rate-limiting.
//!
//! ## Usage
//! ```toml
//! [dependencies]
//! public-ip-address = { version = "0.2" }
//! ```
//! ## Example
//! ```rust
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let result = public_ip_address::perform_lookup(None)?;
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

use log::{debug, trace, warn};
use std::net::IpAddr;

use cache::ResponseCache;
use error::{Error, Result};
use lookup::{error::LookupError, LookupProvider, LookupService, Parameters};
use response::LookupResponse;

pub mod cache;
pub mod error;
pub mod lookup;
pub mod response;

/// Performs a lookup using a predefined list of `LookupProvider`s and caches the result.
///
/// This function performs a lookup using a predefined list of `LookupProvider`s. The list includes
/// `IpInfo`, `IpWhoIs`, `MyIp`, and `FreeIpApi`. The result of the lookup is cached locally for 5 seconds.
/// If a subsequent request is made within 2 seconds, the cached result is returned.
///
/// # Arguments
///
/// * `target` - Target address for the lookup, `None` will look up the current public address.
///
/// # Example
///
/// ```
/// match public_ip_address::perform_lookup(None) {
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
pub fn perform_lookup(target: Option<IpAddr>) -> Result<LookupResponse> {
    perform_cached_lookup_with(
        vec![
            (LookupProvider::IpInfo, None),
            (LookupProvider::IpWhoIs, None),
            (LookupProvider::MyIp, None),
            (LookupProvider::FreeIpApi, None),
        ],
        target,
        Some(5),
        false,
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
/// * `providers` - A vector of `LookupProvider`s and their `Parameters` to use for the lookup.
/// * `target` - Target address for the lookup, `None` will look up the current public address.
///
/// # Example
///
/// ```rust
/// use public_ip_address::lookup::LookupProvider;
///
/// let providers = vec![
///     // List of providers to use for the lookup
///     // (LookupProvider::IpWhoIs, Some(Parameters::new(apikey)))
/// ];
///
/// match public_ip_address::perform_lookup_with(providers, None) {
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
pub fn perform_lookup_with(
    providers: Vec<(LookupProvider, Option<Parameters>)>,
    target: Option<IpAddr>,
) -> Result<LookupResponse> {
    let mut errors = Vec::new();
    if providers.is_empty() {
        return Err(Error::LookupError(LookupError::GenericError(
            "No providers given".to_string(),
        )));
    }

    for (provider, param) in providers {
        debug!("Performing lookup with provider {}", &provider);
        let response = LookupService::new(provider, param).lookup(target);
        if let Ok(response) = response {
            trace!("Successful response from provider");
            return Ok(response);
        }
        warn!("Provider failed to perform lookup");
        errors.push(response.unwrap_err());
    }

    // if we reach here no responses were found
    warn!("No responses from providers");
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
/// If `cache_expire_time` is `0`, then the cache is expired immediately after the request.
///
/// # Arguments
///
/// * `providers` - A vector of `LookupProvider`s and their `Parameters` to use for the lookup.
/// * `target` - Target address for the lookup, `None` will look up the current public address.
/// * `cache_expire_time` - An `Option` containing the number of seconds before the cache expires. If `None`,
///   the cache never expires. If `0`, the cache expires immediately after the request.
/// * `flush` - A `bool` indicating whether to force the cache to flush and make a new request.
///
/// # Example
///
/// ```rust
/// use public_ip_address::lookup::LookupProvider;
///
/// let providers = vec![
///     // List of providers to use for the lookup
///     // (LookupProvider::IpWhoIs, Some(Parameters::new(apikey)))
/// ];
/// let expire_time = Some(60); // Cache expires after 60 seconds
/// let flush = false; // Do not force cache flush
///
/// match public_ip_address::perform_cached_lookup_with(providers, None, expire_time, flush) {
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
    providers: Vec<(LookupProvider, Option<Parameters>)>,
    target: Option<IpAddr>,
    ttl: Option<u64>,
    flush: bool,
) -> Result<LookupResponse> {
    let cached_file = ResponseCache::load();
    // load the cache if it exists
    let mut cache = match cached_file {
        Ok(cache) => {
            // check if we are looking for a specific target
            if let Some(target) = target {
                if !cache.target_is_expired(&target) && !flush {
                    if let Some(target) = cache.lookup_address.get(&target) {
                        trace!("Using cached value");
                        return Ok(target.response.to_owned());
                    }
                }
            } else if !cache.current_is_expired() && !flush {
                if let Some(current) = cache.current_address {
                    trace!("Using cached value");
                    return Ok(current.response);
                }
            }
            cache
        }
        // no cache file, create a new cache
        Err(_) => ResponseCache::default(),
    };

    trace!("Performing new lookup");
    // no cache or it's too old, make a new request.
    match perform_lookup_with(providers, target) {
        Ok(result) => {
            if let Some(target) = target {
                cache.update_target(target, &result, ttl);
            } else {
                cache.update_current(&result, ttl);
            }
            cache.save()?;
            Ok(result)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::net::IpAddr;

    fn clear_cache() {
        _ = ResponseCache::default().delete();
    }

    fn ip(ip: &str) -> IpAddr {
        ip.parse().unwrap()
    }

    #[test]
    fn test_perform_lookup() {
        let response = perform_lookup_with(
            vec![(LookupProvider::Mock("1.1.1.1".to_string()), None)],
            None,
        );
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            ip("1.1.1.1"),
            "IP address not matching"
        );
    }

    #[test]
    fn test_perform_lookup_target() {
        let response = perform_lookup_with(
            vec![(LookupProvider::Mock("1.1.1.1".to_string()), None)],
            Some(ip("8.8.8.8")),
        );
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap().ip,
            ip("8.8.8.8"),
            "IP address not matching"
        );
    }

    #[test]
    #[serial]
    fn test_perform_lookup_cached() {
        clear_cache();
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("11.1.1.1".to_string()), None)],
            None,
            Some(1),
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("11.1.1.1"),
            "IP address not matching"
        );
        clear_cache();
    }

    #[test]
    #[serial]
    fn test_perform_lookup_cached_force_expire() {
        clear_cache();
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("21.1.1.1".to_string()), None)],
            None,
            None,
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("21.1.1.1"),
            "IP address not matching"
        );
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("22.2.2.2".to_string()), None)],
            None,
            Some(1),
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("21.1.1.1"),
            "Non expiring cache should be used"
        );
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("23.3.3.3".to_string()), None)],
            None,
            Some(1),
            true,
        );
        // the old cache should be flushed
        assert_eq!(
            response.unwrap().ip,
            ip("23.3.3.3"),
            "The old cache should be flushed"
        );
        clear_cache();
    }

    #[test]
    #[serial]
    fn test_perform_lookup_cached_expired() {
        clear_cache();
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("1.1.1.1".to_string()), None)],
            None,
            Some(1),
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("1.1.1.1"),
            "IP address not matching"
        );
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("2.2.2.2".to_string()), None)],
            None,
            Some(2),
            false,
        );
        // the old cache should be returned
        assert_eq!(
            response.unwrap().ip,
            ip("1.1.1.1"),
            "The old cache should be returned"
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("3.3.3.3".to_string()), None)],
            None,
            Some(0),
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("3.3.3.3"),
            "Cached value should expire"
        );

        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("4.4.4.4".to_string()), None)],
            None,
            Some(1),
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("4.4.4.4"),
            "Cached value should expire"
        );
        let response = perform_cached_lookup_with(
            vec![(LookupProvider::Mock("5.5.5.5".to_string()), None)],
            None,
            Some(1),
            false,
        );
        assert_eq!(
            response.unwrap().ip,
            ip("4.4.4.4"),
            "Cached value should be used"
        );
        clear_cache();
    }
}
