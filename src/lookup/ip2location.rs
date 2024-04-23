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
    latitude: Option<f64>,
    longitude: Option<f64>,
    zip_code: Option<String>,
    time_zone: Option<String>,
    asn: Option<String>,
    #[serde(rename = "as")]
    as_name: Option<String>,
    is_proxy: Option<bool>,
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
            LookupProvider::Ip2Location,
        );
        response.country = self.country_name;
        response.country_code = self.country_code;
        response.region = self.region_name;
        response.postal_code = self.zip_code;
        response.city = self.city_name;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        response.asn_org = self.as_name;
        response.asn = self.asn;
        response.is_proxy = self.is_proxy;

        response
    }
}

pub struct Ip2Location;

#[async_trait::async_trait]
impl Provider for Ip2Location {
    #[inline]
    fn get_endpoint(&self, key: &Option<String>, target: &Option<IpAddr>) -> String {
        let key = match key {
            Some(k) => format!("?key={}", k),
            None => "".to_string(),
        };
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("&ip={}", t),
            None => "".to_string(),
        };
        format!("https://api.ip2location.io/{}{}", key, target)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = Ip2LocationResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Ip2Location
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

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        let service = Box::new(Ip2Location);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ip2Location: {:#?}", result);

        let response = Ip2LocationResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[tokio::test]
    #[ignore]
    async fn test_request_with_key() {
        use std::env;
        let key = env::var("IP2LOCATION_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(Ip2Location);
        let result = service.get_client(key, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ip2Location: {:#?}", result);

        let response = Ip2LocationResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[tokio::test]
    #[ignore]
    async fn test_request_with_key_for_target() {
        use std::env;
        let key = env::var("IP2LOCATION_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let target = "8.8.8.8".parse::<IpAddr>().ok();
        let service = Box::new(Ip2Location);
        let result = service.get_client(key, target).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ip2Location: {:#?}", result);

        let response = Ip2LocationResponse::parse(result).unwrap();
        assert_eq!(response.ip, "8.8.8.8", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(lookup.ip, target.unwrap(), "IP address not matching");
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
