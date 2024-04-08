//! <https://ip2location.io> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://www.ip2location.io/ip2location-documentation
#[derive(Serialize, Deserialize, Debug)]
pub struct Ip2LocationResponse {
    ip: String,
    country_code: Option<String>,
    country_name: Option<String>,
    region_name: Option<String>,
    city_name: Option<String>,
    latitude: Option<String>,
    longitude: Option<String>,
    zip_code: Option<String>,
    time_zone: Option<String>,
    is_eu: Option<bool>,
    asn: Option<String>,
    #[serde(rename = "as")]
    as_name: Option<String>,
    is_proxy: Option<String>,
}

impl Ip2LocationResponse {
    pub fn parse(input: String) -> Result<Ip2LocationResponse> {
        let deserialized: Ip2LocationResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::Ip2Location(None),
        );
        response.continent = self.continent_name;
        response.country = self.country_name;
        response.country_code = self.country_code2;
        response.region = self.state_prov;
        response.postal_code = self.zipcode;
        response.city = self.city;
        if let Some(lat) = self.latitude {
            response.latitude = lat.parse().ok();
        }
        if let Some(lon) = self.longitude {
            response.longitude = lon.parse().ok();
        }
        if let Some(timezone) = self.time_zone {
            response.time_zone = timezone.name;
        }
        response.hostname = self.hostname;
        response.asn_org = self.organization;
        response.asn = self.isp;

        response
    }
}

pub struct Ip2Location {
    key: Option<String>,
}

impl Ip2Location {
    /// Create a new Ip2Location instance with an API key
    pub fn new(key: Option<String>) -> Ip2Location {
        Ip2Location { key }
    }
}

impl Provider for Ip2Location {
    fn make_api_request(&self) -> Result<String> {
        let key = match self.key.as_ref() {
            Some(k) => format!("/?key={}", k),
            None => "".to_string(),
        };
        let endpoint = format!("https://api.ip2location.io{}", key);
        let response = reqwest::blocking::get(endpoint);
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = Ip2LocationResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Ip2Location(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
	"ip":"8.8.8.8",
	"country_code":"US",
	"country_name":"United States of America",
	"region_name":"California",
	"city_name":"Mountain View",
	"latitude":37.405992,
	"longitude":-122.078515,
	"zip_code":"94043",
	"time_zone":"-07:00",
	"asn":"15169",
	"as":"Google LLC",
	"is_proxy":false
}
"#;

    #[test]
    #[ignore]
    fn test_request() {
        let service = Box::new(Ip2Location::new(None));
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ip2Location: {:#?}", result);

        let response = Ip2LocationResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    #[ignore]
    fn test_request_with_key() {
        use std::env;
        let key = env::var("IP2LOCATION_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(Ip2Location::new(key));
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ip2Location: {:#?}", result);

        let response = Ip2LocationResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = Ip2LocationResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "8.8.8.8", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "8.8.8.8".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
