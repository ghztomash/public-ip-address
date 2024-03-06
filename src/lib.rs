use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::time::{Duration, SystemTime};
use std::error;

const CACHE_TIME: u64 = 2;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    ip: String,
    ip_decimal: u32,
    country: Option<String>,
    country_iso: Option<String>,
    country_eu: Option<bool>,
    region_name: Option<String>,
    region_code: Option<String>,
    metro_code: Option<String>,
    zip_code: Option<String>,
    city: Option<String>,
    latitude: Option<f32>,
    longitude: Option<f32>,
    time_zone: Option<String>,
    asn: Option<String>,
    asn_org: Option<String>,
    hostname: Option<String>,
    user_agent: Option<String>,
}

impl Response {
    pub fn parse(input: String) -> Result<Response> {
        let deserialized: Response = serde_json::from_str(&input)?;
        Ok(deserialized)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Cache {
    response: Response,
    response_time: SystemTime,
}

impl Cache {
    #[cfg(unix)]
    const CACHE_PATH: &'static str = "/private/tmp/public-ip-cache.txt";
    #[cfg(not(unix))]
    const CACHE_PATH: &'static str = "public-ip-cache.txt";

    fn new(response: Response) -> Cache {
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

fn make_api_request() -> Result<String> {
    let response = reqwest::blocking::get("http://ifconfig.co/json")?.text()?;
    Ok(response)
}

pub fn get_response() -> Result<Response> {
    let cached = Cache::load();
    if let Ok(cache) = cached {
        let difference = SystemTime::now()
            .duration_since(cache.response_time)?;
        println!("Difference: {:?}", difference);
        if difference <= Duration::from_secs(CACHE_TIME) {
            println!("Using cache");
            return Ok(cache.response);
        }
    }
    println!("Making new request");
    // no chache or it's too old, make a new request.
    if let Ok(result) = make_api_request() {
        let response = Response::parse(result)?;
        let cache = Cache::new(response);
        cache.save()?;
        return Ok(cache.response);
    }
    Err("Error getting response".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = "{\n \"ip\": \"1.1.1.1\",\n \"ip_decimal\": 16843009\n}";

    #[test]
    fn test_request() {
        let result = make_api_request();
        assert!(result.is_ok(), "Failed getting result");
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("{:#?}", result);
    }

    #[test]
    fn test_parse() {
        let response = Response::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
    }

    #[test]
    fn test_cache() {
        let response = Response::parse(TEST_INPUT.to_string()).unwrap();
        let cache = Cache::new(response);
        cache.save().unwrap();
        let cached = Cache::load().unwrap();
        assert_eq!(cached.response.ip, "1.1.1.1", "IP address not matching");
        assert_eq!(
            cache.response.ip_decimal, cached.response.ip_decimal,
            "IP address not matching"
        );
    }

    #[test]
    fn test_get_response() {
        let response = get_response();
        println!("{:#?}", response);
        assert!(response.is_ok());
    }
}
