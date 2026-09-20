//! # 🔎 Public IP Address Lookup and Geolocation Information
//!
//! `public-ip-address` is a simple, easy-to-use Rust library for performing public IP lookups from various services.
//! It provides a unified interface to fetch public IP address and geolocation information from multiple providers.
//!
//! Arbitrary IP address lookup and access API keys are supported for certain providers.
//!
//! The library provides an asynchronous and blocking interfaces to make it easy to integrate with other `async` codebase.
//!
//! The library also includes caching functionality to improve performance for repeated lookups and minimaze rate-limiting.
//! The cache file can be encrypted when enabled through the `encryption` feature flag for additional privacy.
//!
//! ## Usage
//! ```toml
//! [dependencies]
//! public-ip-address = { version = "0.4" }
//! ```
//! ## Example
//! ```rust
//! use std::error::Error;
//! #[cfg_attr(not(feature = "blocking"), tokio::main)]
//! #[maybe_async::maybe_async]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let result = public_ip_address::perform_lookup(None).await?;
//!     println!("{}", result);
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//! - Unified interface for multiple IP lookup providers
//! - Caching of lookup results to improve performance
//! - Customizable cache expiration time
//! - Optional caller-supplied `reqwest` client, for callers that need to control
//!   the transport (proxy or `no_proxy()`, timeouts, a shared connection pool)
//!
//! For more details, please refer to the API documentation.

#![warn(missing_docs)]

use log::{debug, trace, warn};
use std::net::IpAddr;

use cache::ResponseCache;
use error::{Error, Result};
use lookup::{error::LookupError, Client, LookupProvider, LookupService, Parameters};
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
/// ```rust
/// # use std::error::Error;
/// # #[cfg_attr(not(feature = "blocking"), tokio::main)]
/// # #[maybe_async::maybe_async]
/// # async fn main() -> Result<(), Box<dyn Error>> {
/// match public_ip_address::perform_lookup(None).await {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or an `Error` if the lookup or caching failed.
#[maybe_async::maybe_async]
pub async fn perform_lookup(target: Option<IpAddr>) -> Result<LookupResponse> {
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
    .await
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
/// # use std::error::Error;
/// # #[cfg_attr(not(feature = "blocking"), tokio::main)]
/// # #[maybe_async::maybe_async]
/// # async fn main() -> Result<(), Box<dyn Error>> {
///
/// let providers = vec![
///     // List of providers to use for the lookup
///     // (LookupProvider::IpWhoIs, Some(Parameters::new(apikey)))
/// ];
///
/// match public_ip_address::perform_lookup_with(providers, None).await {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or a `LookupError` containing a list of all errors received.
#[maybe_async::maybe_async]
pub async fn perform_lookup_with(
    providers: Vec<(LookupProvider, Option<Parameters>)>,
    target: Option<IpAddr>,
) -> Result<LookupResponse> {
    perform_lookup_impl(None, providers, target).await
}

/// Performs a lookup using a list of providers, sending every request with `client`.
///
/// This behaves exactly like [`perform_lookup_with`], except that each provider
/// request is issued by the client the caller supplies instead of a default one.
/// Use it when the transport matters: a proxy, an explicit `no_proxy()` so the
/// request is never tunneled, a custom timeout, or a shared connection pool.
///
/// # Arguments
///
/// * `client` - The client every provider request is sent with.
/// * `providers` - A vector of `LookupProvider`s and their `Parameters` to use for the lookup.
/// * `target` - Target address for the lookup, `None` will look up the current public address.
///
/// # Example
///
/// ```rust
/// use public_ip_address::lookup::{Client, LookupProvider};
///
/// # use std::error::Error;
/// # #[cfg_attr(not(feature = "blocking"), tokio::main)]
/// # #[maybe_async::maybe_async]
/// # async fn main() -> Result<(), Box<dyn Error>> {
/// // A client that ignores every proxy, including the ones configured in the
/// // environment or by the operating system.
/// let client = Client::builder()
///     .no_proxy()
///     .timeout(std::time::Duration::from_secs(10))
///     .build()?;
///
/// let providers = vec![
///     // List of providers to use for the lookup
///     // (LookupProvider::IpWhoIs, Some(Parameters::new(apikey)))
/// ];
///
/// match public_ip_address::perform_lookup_with_client(&client, providers, None).await {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Note
///
/// There is deliberately no cached counterpart to this function. A caller that wants both
/// a custom client and a cache can drive [`cache::ResponseCache`] directly around this call.
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or a `LookupError` containing a list of all errors received.
#[maybe_async::maybe_async]
pub async fn perform_lookup_with_client(
    client: &Client,
    providers: Vec<(LookupProvider, Option<Parameters>)>,
    target: Option<IpAddr>,
) -> Result<LookupResponse> {
    perform_lookup_impl(Some(client), providers, target).await
}

/// Shared body of `perform_lookup_with` and `perform_lookup_with_client`.
///
/// `client` of `None` keeps the historic behaviour: every provider builds its
/// own default client through `Provider::get_client`.
#[maybe_async::maybe_async]
async fn perform_lookup_impl(
    client: Option<&Client>,
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
        let service = match client {
            Some(client) => LookupService::with_client(provider, param, client),
            None => LookupService::new(provider, param),
        };
        let response = service.lookup(target).await;
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
        "No responses from providers: {errors:?}"
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
/// # use std::error::Error;
/// # #[cfg_attr(not(feature = "blocking"), tokio::main)]
/// # #[maybe_async::maybe_async]
/// # async fn main() -> Result<(), Box<dyn Error>> {
/// let providers = vec![
///     // List of providers to use for the lookup
///     // (LookupProvider::IpWhoIs, Some(Parameters::new(apikey)))
/// ];
/// let expire_time = Some(60); // Cache expires after 60 seconds
/// let flush = false; // Do not force cache flush
///
/// match public_ip_address::perform_cached_lookup_with(providers, None, expire_time, flush).await {
///     Ok(response) => {
///         // Handle successful response
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// * A `Result` containing either a successful `LookupResponse` or an `Error` if the lookup or caching failed.
#[maybe_async::maybe_async]
pub async fn perform_cached_lookup_with(
    providers: Vec<(LookupProvider, Option<Parameters>)>,
    target: Option<IpAddr>,
    ttl: Option<u64>,
    flush: bool,
) -> Result<LookupResponse> {
    let cached_file = ResponseCache::load(None);
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
    match perform_lookup_with(providers, target).await {
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
