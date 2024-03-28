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
use lookup::{LookupProvider, LookupService};
use serde::{Deserialize, Serialize};
use std::fmt;
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
    /// Create a new lookup response.
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

impl fmt::Display for LookupResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "IP: {}", self.ip)?;
        if let Some(continent) = &self.continent {
            writeln!(f, "Continent: {}", continent)?;
        }
        if let Some(country) = &self.country {
            write!(f, "Country: {}", country)?;
        }
        if let Some(country_code) = &self.country_code {
            writeln!(f, " ({})", country_code)?;
        } else {
            writeln!(f)?;
        }
        if let Some(region) = &self.region {
            write!(f, "Region: {}", region)?;
        }
        if let Some(region_code) = &self.region_code {
            writeln!(f, " ({})", region_code)?;
        } else {
            writeln!(f)?;
        }
        if let Some(postal_code) = &self.postal_code {
            writeln!(f, "Postal code: {}", postal_code)?;
        }
        if let Some(city) = &self.city {
            writeln!(f, "City: {}", city)?;
        }
        if let Some(latitude) = &self.latitude {
            write!(f, "Coordinates: {}", latitude)?;
        }
        if let Some(longitude) = &self.longitude {
            writeln!(f, ", {}", longitude)?;
        } else {
            writeln!(f)?;
        }
        if let Some(time_zone) = &self.time_zone {
            writeln!(f, "Time zone: {}", time_zone)?;
        }
        if let Some(asn_org) = &self.asn_org {
            write!(f, "Organization: {}", asn_org)?;
        }
        if let Some(asn) = &self.asn {
            writeln!(f, " ({})", asn)?;
        } else {
            writeln!(f)?;
        }
        if let Some(hostname) = &self.hostname {
            writeln!(f, "Hostname: {}", hostname)?;
        }
        if let Some(proxy) = &self.proxy {
            writeln!(f, "Proxy: {}", proxy)?;
        }
        write!(f, "Provider: {}", self.provider)?;

        Ok(())
    }
}

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
