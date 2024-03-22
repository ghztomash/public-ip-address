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
use error::Result;
use lookup::{LookupProvider, LookupService};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

pub mod cache;
pub mod error;
pub mod lookup;

const DEFAULT_CACHE_TIME: u64 = 2;

/// Lookup response containing the public IP address.
/// As well as additional lookup information like country, city, hostname etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LookupResponse {
    pub ip: String,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub time_zone: Option<String>,
    pub asn: Option<String>,
    pub asn_org: Option<String>,
    pub hostname: Option<String>,
    pub proxy: Option<bool>,
    pub provider: LookupProvider,
}

impl LookupResponse {
    pub fn new(ip: String, provider: LookupProvider) -> Self {
        LookupResponse {
            ip,
            continent: None,
            country: None,
            country_code: None,
            region: None,
            region_code: None,
            postal_code: None,
            city: None,
            latitude: None,
            longitude: None,
            time_zone: None,
            asn: None,
            asn_org: None,
            hostname: None,
            proxy: None,
            provider,
        }
    }
}

impl std::fmt::Display for LookupResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

/// Perform a simple lookup.
/// This calls `perform_cached_lookup_with()` with default values.
pub fn perform_lookup() -> Result<LookupResponse> {
    perform_cached_lookup_with(LookupProvider::IfConfig, None)
}

/// Performs lookup with a specific service provider.
pub fn perform_lookup_with(provider: LookupProvider) -> Result<LookupResponse> {
    LookupService::new(provider).make_request()
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
        println!("Difference: {:?}", difference);
        if difference <= Duration::from_secs(cache_time.unwrap_or(DEFAULT_CACHE_TIME)) {
            println!("Using cache");
            return Ok(cache.response);
        }
    }

    println!("Making new request");
    let service = LookupService::new(provider);
    // no cache or it's too old, make a new request.
    match service.make_request() {
        Ok(result) => {
            let cache = ResponseCache::new(result);
            cache.save()?;
            Ok(cache.response)
        }
        Err(e) => Err(format!("Error getting lookup response: {}", e).into()),
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
}
