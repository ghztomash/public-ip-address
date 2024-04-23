//! <https://iplocate.io> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://iplocate.docs.apiary.io/
#[derive(Serialize, Deserialize, Debug)]
pub struct IpLocateIoResponse {
    ip: String,
    country: Option<String>,
    country_code: Option<String>,
    is_eu: Option<bool>,
    city: Option<String>,
    continent: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    time_zone: Option<String>,
    postal_code: Option<String>,
    subdivision: Option<String>,
    org: Option<String>,
    asn: Option<String>,
    threat: Option<Threat>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Threat {
    is_proxy: Option<bool>,
}

impl IpLocateIoResponse {
    pub fn parse(input: String) -> Result<IpLocateIoResponse> {
        let deserialized: IpLocateIoResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpLocateIo,
        );
        response.country = self.country;
        response.continent = self.continent;
        response.country_code = self.country_code;
        response.region = self.subdivision;
        response.postal_code = self.postal_code;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        response.asn_org = self.org;
        response.asn = self.asn;
        if let Some(threat) = self.threat {
            response.is_proxy = threat.is_proxy;
        }
        response
    }
}

pub struct IpLocateIo;

#[async_trait::async_trait]
impl Provider for IpLocateIo {
    #[inline]
    fn get_endpoint(&self, key: &Option<String>, target: &Option<IpAddr>) -> String {
        let key = match key {
            Some(k) => format!("?apikey={}", k),
            None => "".to_string(),
        };
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("{}/", t),
            None => "".to_string(),
        };
        format!("https://www.iplocate.io/api/lookup{}/json{}", target, key)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpLocateIoResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpLocateIo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "asn": "AS6185",
  "city": "Cupertino",
  "continent": "North America",
  "country": "United States",
  "country_code": "US",
  "ip": "1.1.1.1",
  "org": "Apple Inc.",
  "latitude": 37.3042,
  "longitude": -122.0946,
  "postal_code": "95014",
  "subdivision": "California",
  "time_zone": "America/Los_Angeles"
}
"#;

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        let service = Box::new(IpLocateIo);
        let result = service.make_api_request(None, None).await;
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpLocateIo: {:#?}", result);
        let response = IpLocateIoResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpLocateIoResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
