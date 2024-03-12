use error::Result;
use lookup::mock::Mock;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::time::{Duration, SystemTime};

mod error;
mod lookup;

const CACHE_TIME: u64 = 2;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupResponse {
    ip: String,
    country: Option<String>,
    country_iso: Option<String>,
    country_eu: Option<bool>,
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
}

impl LookupResponse {
    pub fn new(ip: String) -> Self {
        LookupResponse {
            ip,
            country: None,
            country_iso: None,
            country_eu: None,
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Cache {
    response: LookupResponse,
    response_time: SystemTime,
}

impl Cache {
    #[cfg(unix)]
    const CACHE_PATH: &'static str = "/private/tmp/public-ip-cache.txt";
    #[cfg(not(unix))]
    const CACHE_PATH: &'static str = "public-ip-cache.txt";

    fn new(response: LookupResponse) -> Cache {
        let cache = Cache {
            response,
            response_time: SystemTime::now(),
        };
        cache
    }

    fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string(self)?;
        let mut file = File::create(Self::CACHE_PATH)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn load() -> Result<Cache> {
        let mut file = File::open(Self::CACHE_PATH)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let deserialized: Cache = serde_json::from_str(&contents)?;
        Ok(deserialized)
    }
}

pub fn get_response() -> Result<LookupResponse> {
    let cached = Cache::load();
    if let Ok(cache) = cached {
        let difference = SystemTime::now().duration_since(cache.response_time)?;
        println!("Difference: {:?}", difference);
        if difference <= Duration::from_secs(CACHE_TIME) {
            println!("Using cache");
            return Ok(cache.response);
        }
    }
    println!("Making new request");
    // no chache or it's too old, make a new request.
    let service = lookup::Service::new(Box::new(Mock { ip: "1.1.1.1" }));
    if let Ok(result) = service.make_request() {
        let cache = Cache::new(result);
        cache.save()?;
        return Ok(cache.response);
    }
    Err("Error getting response".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let response = LookupResponse::new("1.1.1.1".to_string());
        let cache = Cache::new(response);
        cache.save().unwrap();
        let cached = Cache::load().unwrap();
        assert_eq!(cached.response.ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_get_response() {
        let response = get_response();
        println!("{:#?}", response);
        assert!(response.is_ok());
    }
}
