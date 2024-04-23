//! <https://ipapi.co> lookup provider

use super::{client::RequestBuilder, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://ipapi.co/api/
#[derive(Serialize, Deserialize, Debug)]
pub struct IpApiCoResponse {
    ip: String,
    version: Option<String>,
    city: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    country_name: Option<String>,
    country_code: Option<String>,
    continent_code: Option<String>,
    in_eu: Option<bool>,
    postal: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<String>,
    currency: Option<String>,
    isp: Option<String>,
    asn: Option<String>,
    org: Option<String>,
    hostname: Option<String>,
}

impl IpApiCoResponse {
    pub fn parse(input: String) -> Result<IpApiCoResponse> {
        let deserialized: IpApiCoResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpApiCo,
        );
        response.country = self.country_name;
        response.country_code = self.country_code;
        response.region = self.region;
        response.postal_code = self.postal;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.timezone;
        response.asn_org = self.org;
        response.asn = self.asn;
        response.hostname = self.hostname;
        response
    }
}

pub struct IpApiCo;

#[async_trait::async_trait]
impl Provider for IpApiCo {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("{}/", t),
            None => "".to_string(),
        };
        format!("https://ipapi.co/{}json", target)
    }

    #[inline]
    fn add_auth(&self, request: RequestBuilder, _key: &Option<String>) -> RequestBuilder {
        request.header("User-Agent", "nil")
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpApiCoResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpApiCo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
    "ip": "1.1.1.1",
    "city": "San Francisco",
    "region": "California",
    "region_code": "CA",
    "country": "US",
    "country_name": "United States",
    "continent_code": "NA",
    "in_eu": false,
    "postal": "94107",
    "latitude": 37.7697,
    "longitude": -122.3933,
    "timezone": "America/Los_Angeles",
    "utc_offset": "-0700",
    "country_calling_code": "+1",
    "currency": "USD",
    "languages": "en-US,es-US,haw,fr",
    "asn": "AS36692",
    "org": "OpenDNS, LLC"
}
"#;

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        let service = Box::new(IpApiCo);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpApiCo: {:#?}", result);
        let response = IpApiCoResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpApiCoResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
