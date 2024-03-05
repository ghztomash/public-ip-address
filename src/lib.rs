use std::time::SystemTime;
use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use reqwest::{Error};

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    ip: String,
    ip_decimal: u32,
    country: String,
    country_iso: String,
    country_eu: bool,
    region_name: String,
    region_code: String,
    zip_code: String,
    city: String,
    latitude: f32,
    longitude: f32,
    time_zone: String,
    asn: String,
    asn_org: String,
}

impl Response {
    fn parse(input: String) -> Result<Response, Error> {
        let deserialized: Response = serde_json::from_str(&input).unwrap();
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
        let cache = Cache{response, response_time: SystemTime::now()};
        cache.save().unwrap();
        cache
    }

    fn save(&self) -> std::io::Result<()> {
        let serialized = serde_json::to_string(self).unwrap();
        let mut file = File::create(Self::CACHE_PATH)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn load() -> Option<Cache> {
        let mut file = File::open(Self::CACHE_PATH).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let deserialized: Cache = serde_json::from_str(&contents).unwrap();
        Some(deserialized)
    }
}

fn make_api_request() -> Result<String, Error> {
    let response = reqwest::blocking::get("http://ifconfig.co/json")?.text()?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = "{\n  \"ip\": \"1.1.1.1\",\n  \"ip_decimal\": 16843009,\n  \"country\": \"Germany\",\n  \"country_iso\": \"DE\",\n  \"country_eu\": true,\n  \"region_name\": \"Hesse\",\n  \"region_code\": \"HE\",\n  \"zip_code\": \"60326\",\n  \"city\": \"Frankfurt am Main\",\n  \"latitude\": 50.1049,\n  \"longitude\": 8.6295,\n  \"time_zone\": \"Europe/Berlin\",\n  \"asn\": \"AS9009\",\n  \"asn_org\": \"M247 Europe SRL\"\n}";

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
        let cached = Cache::load().unwrap();
        assert_eq!(cached.response.ip, "1.1.1.1", "IP address not matching");
        assert_eq!(cache.response.ip_decimal, cached.response.ip_decimal, "IP address not matching");
    }
}
