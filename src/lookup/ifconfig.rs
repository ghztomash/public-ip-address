use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse, Result,
};
use serde::{Deserialize, Serialize};

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
    metro_code: Option<String>,
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

    pub fn convert(&self) -> LookupResponse {
        let mut response = LookupResponse::new(self.ip.clone());
        response.country = self.country.clone();
        response.country_iso = self.country_iso.clone();
        if self.country_eu.unwrap_or(false) {
            response.continent = Some("Europe".to_string());
        }
        response.region_name = self.region_name.clone();
        response.region_code = self.region_code.clone();
        response.metro_code = self.metro_code.clone();
        response.zip_code = self.zip_code.clone();
        response.city = self.city.clone();
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone.clone();
        response.asn = self.asn.clone();
        response.asn_org = self.asn_org.clone();
        response.hostname = self.hostname.clone();
        response
    }
}

pub struct IfConfig;
impl Provider for IfConfig {
    fn make_api_request(&self) -> Result<String> {
        let response = reqwest::blocking::get("http://ifconfig.co/json");
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IfConfigResponse::parse(json)?;
        Ok(response.convert())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = "{\n \"ip\": \"1.1.1.1\",\n \"ip_decimal\": 16843009\n}";

    #[test]
    fn test_request() {
        let service = Box::new(IfConfig);
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result");
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ifconfig: {:#?}", result);
    }

    #[test]
    fn test_parse() {
        let response = IfConfigResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
    }
}
