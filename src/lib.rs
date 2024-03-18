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
    ip: String,
    country: Option<String>,
    country_iso: Option<String>,
    continent: Option<String>,
    region_name: Option<String>,
    region_code: Option<String>,
    metro_code: Option<String>,
    zip_code: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    time_zone: Option<String>,
    asn: Option<String>,
    asn_org: Option<String>,
    hostname: Option<String>,
    proxy: Option<bool>,
    provider: LookupProvider,
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

pub fn lookup() -> Result<LookupResponse> {
    let service = LookupService::new(LookupProvider::IfConfig);
    lookup_with_service_cache(service, None)
}

pub fn lookup_with_service(service: LookupService) -> Result<LookupResponse> {
    match service.make_request() {
        Ok(result) => {
            ResponseCache::new(result.clone()).save()?;
            Ok(result)
        }
        Err(e) => Err(format!("Error getting lookup response: {}", e).into()),
    }
}

pub fn lookup_with_service_cache(
    service: LookupService,
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
        let response = lookup_with_service(LookupService::new(LookupProvider::Mock(
            "1.1.1.1".to_string(),
        )));
        assert!(response.is_ok());
        assert_eq!(response.unwrap().ip, "1.1.1.1", "IP address not matching");
    }
}
