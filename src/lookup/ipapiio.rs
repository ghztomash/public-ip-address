//! <https://ip-api.io> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://ip-api.io/
#[derive(Serialize, Deserialize, Debug)]
pub struct IpApiIoResponse {
    ip: String,
    city: Option<String>,
    country_code: Option<String>,
    country_name: Option<String>,
    currency: Option<String>,
    is_in_european_union: Option<bool>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    organisation: Option<String>,
    region_code: Option<String>,
    region_name: Option<String>,
    #[serde(rename = "suspiciousFactors")]
    suspicious_factors: Option<SuspiciousFactors>,
    time_zone: Option<String>,
    zip_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SuspiciousFactors {
    is_proxy: Option<bool>,
    is_spam: Option<bool>,
    is_suspicious: Option<bool>,
    is_tor_node: Option<bool>,
}

impl IpApiIoResponse {
    pub fn parse(input: String) -> Result<IpApiIoResponse> {
        let deserialized: IpApiIoResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpApiIo,
        );
        response.country = self.country_name;
        response.country_code = self.country_code;
        response.region = self.region_name;
        response.postal_code = self.zip_code;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        response.asn_org = self.organisation;
        if let Some(suspicious) = self.suspicious_factors {
            response.is_proxy = suspicious.is_proxy;
        }
        if self.is_in_european_union.unwrap_or(false) {
            response.continent = Some("Europe".to_string());
        }
        response
    }
}

pub struct IpApiIo;
impl Provider for IpApiIo {
    fn make_api_request(&self, key: Option<String>, target: Option<IpAddr>) -> Result<String> {
        let client = reqwest::blocking::Client::new();
        let response = client.get("https://ip-api.io/json/").send();
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpApiIoResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpApiIo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "callingCode": "1",
  "city": "",
  "countryCapital": "Washington D.C.",
  "country_code": "US",
  "country_name": "United States",
  "currency": "USD,USN,USS",
  "currencySymbol": "$,$",
  "emojiFlag": "🇺🇸",
  "flagUrl": "https://ip-api.io/images/flags/us.svg",
  "ip": "1.1.1.1",
  "is_in_european_union": false,
  "latitude": 37.751,
  "longitude": -97.822,
  "metro_code": 0,
  "organisation": "GOOGLE",
  "region_code": "",
  "region_name": "",
  "suspiciousFactors": {
    "isProxy": false,
    "isSpam": false,
    "isSuspicious": false,
    "isTorNode": false
  },
  "time_zone": "America/Chicago",
  "zip_code": ""
}
"#;

    #[test]
    #[ignore]
    fn test_request() {
        let service = Box::new(IpApiIo);
        let result = service.make_api_request(None, None);
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpApiIo: {:#?}", result);
        let response = IpApiIoResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpApiIoResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
