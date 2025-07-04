//! <https://my-ip.io> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider, ProviderResponse},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://www.my-ip.io/api-usage>
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MyIpResponse {
    success: bool,
    ip: String,
    #[serde(rename = "type")]
    ip_type: Option<String>,
    country: Option<Country>,
    region: Option<String>,
    city: Option<String>,
    location: Option<Location>,
    time_zone: Option<String>,
    asn: Option<Asn>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Country {
    code: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    lat: Option<f64>,
    lon: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Asn {
    number: Option<i64>,
    name: Option<String>,
    network: Option<String>,
}

impl ProviderResponse<MyIpResponse> for MyIpResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::MyIp,
        );
        if let Some(country) = self.country {
            response.country = country.name;
            response.country_code = country.code;
        }
        response.region = self.region;
        response.city = self.city;
        if let Some(location) = self.location {
            response.latitude = location.lat;
            response.longitude = location.lon;
        }
        response.time_zone = self.time_zone;
        if let Some(asn) = self.asn {
            response.asn_org = asn.name;
            if let Some(number) = asn.number {
                response.asn = Some(format!("{number}"));
            }
        }
        response
    }
}

/// MyIp lookup provider
pub struct MyIp;

impl Provider for MyIp {
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        "https://api.my-ip.io/v2/ip.json".to_string()
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = MyIpResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::MyIp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "success": true,
  "ip": "1.1.1.1",
  "type": "IPv4",
  "country": {
    "code": "DE",
    "name": "Germany"
  },
  "region": "Bavaria",
  "city": "Gunzenhausen",
  "location": {
    "lat": 49.1156,
    "lon": 10.7511
  },
  "timeZone": "Europe/Berlin",
  "asn": {
    "number": 24940,
    "name": "ABCD",
    "network": "12.88.0.0/17"
  }
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(MyIp);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("MyIp: {result:#?}");
        let response = MyIpResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[test]
    fn test_parse() {
        let response = MyIpResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
