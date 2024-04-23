//! <https://ifconfig.co> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://github.com/leafcloudhq/echoip/blob/master/http/http.go
#[derive(Serialize, Deserialize, Debug)]
pub struct IfConfigResponse {
    ip: String,
    ip_decimal: u128, // enough to hold ipv6 address
    country: Option<String>,
    country_iso: Option<String>,
    country_eu: Option<bool>,
    region_name: Option<String>,
    region_code: Option<String>,
    metro_code: Option<i64>,
    zip_code: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    time_zone: Option<String>,
    asn: Option<String>,
    asn_org: Option<String>,
    hostname: Option<String>,
    user_agent: Option<String>,
}

impl IfConfigResponse {
    pub fn parse(input: String) -> Result<IfConfigResponse> {
        let deserialized: IfConfigResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IfConfig,
        );
        response.country = self.country;
        response.country_code = self.country_iso;
        if self.country_eu.unwrap_or(false) {
            response.continent = Some("Europe".to_string());
        }
        response.region = self.region_name;
        response.postal_code = self.zip_code;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        response.asn = self.asn;
        response.asn_org = self.asn_org;
        response.hostname = self.hostname;
        response
    }
}

pub struct IfConfig;

#[async_trait::async_trait]
impl Provider for IfConfig {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("?ip={}", t),
            None => "".to_string(),
        };
        format!("http://ifconfig.co/json{}", target)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IfConfigResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IfConfig
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = "{\n \"ip\": \"1.1.1.1\",\n \"ip_decimal\": 16843009\n}";

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        let service = Box::new(IfConfig);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ifconfig: {:#?}", result);
        let response = IfConfigResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IfConfigResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
