use cache::ResponseCache;
use error::Result;
use lookup::{LookupProvider, LookupService};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

pub mod cache;
pub mod error;
pub mod lookup;

const DEFAULT_CACHE_TIME: u64 = 2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LookupResponse {
    pub ip: String,
    pub country: Option<String>,
    pub country_iso: Option<String>,
    pub continent: Option<String>,
    pub region_name: Option<String>,
    pub region_code: Option<String>,
    pub metro_code: Option<String>,
    pub zip_code: Option<String>,
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
            country: None,
            country_iso: None,
            continent: None,
            region_name: None,
            region_code: None,
            metro_code: None,
            zip_code: None,
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

pub fn lookup() -> Result<LookupResponse> {
    lookup_with_service_cache(LookupProvider::IfConfig, None)
}

pub fn lookup_with_service(provider: LookupProvider) -> Result<LookupResponse> {
    let service = LookupService::new(provider);
    match service.make_request() {
        Ok(result) => {
            _ = ResponseCache::new(result.clone()).save();
            Ok(result)
        }
        Err(e) => Err(format!("Error getting lookup response: {}", e).into()),
    }
}

pub fn lookup_with_service_cache(
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
    fn test_get_response() {
        let response = lookup_with_service(LookupProvider::Mock(
            "1.1.1.1".to_string(),
        ));
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }
}
