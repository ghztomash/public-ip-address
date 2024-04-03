//! <https://abstractapi.com> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};

// https://docs.abstractapi.com/ip-geolocation
#[derive(Serialize, Deserialize, Debug)]
pub struct AbstractApiResponse {
    ip_address: String,
    city: Option<String>,
    region: Option<String>,
    region_iso_code: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    country_is_eu: Option<bool>,
    continent: Option<String>,
    continent_code: Option<String>,
    longitude: Option<f64>,
    latitude: Option<f64>,
    security: Option<Security>,
    timezone: Option<Timezone>,
    connection: Option<Connection>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Security {
    is_vpn: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timezone {
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Connection {
    autonomous_system_number: Option<i64>,
    connection_type: Option<String>,
    organization_name: Option<String>,
    isp_name: Option<String>,
}

impl AbstractApiResponse {
    pub fn parse(input: String) -> Result<AbstractApiResponse> {
        let deserialized: AbstractApiResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(self.ip_address, LookupProvider::AbstractApi(None));
        response.country = self.country;
        response.continent = self.continent;
        response.country_code = self.country_code;
        response.region = self.region;
        response.region_code = self.region_iso_code;
        response.postal_code = self.postal_code;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        if let Some(timezone) = self.timezone {
            response.time_zone = timezone.name;
        }
        if let Some(connection) = self.connection {
            response.asn_org = connection.organization_name;
            response.asn = connection.isp_name;
        }
        if let Some(security) = self.security {
            response.proxy = security.is_vpn;
        }

        response
    }
}

pub struct AbstractApi {
    key: Option<String>,
}

impl AbstractApi {
    /// Create a new AbstractApi instance with an API key
    pub fn new(key: Option<String>) -> AbstractApi {
        AbstractApi { key }
    }
}

impl Provider for AbstractApi {
    fn make_api_request(&self) -> Result<String> {
        let endpoint = format!(
            "https://ipgeolocation.abstractapi.com/v1/?api_key={}",
            self.key.as_ref().unwrap_or(&"".to_string())
        );
        let response = reqwest::blocking::get(endpoint);
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = AbstractApiResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::AbstractApi(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
    "ip_address": "1.1.1.1",
    "city": "San Jose",
    "city_geoname_id": 5392171,
    "region": "California",
    "region_iso_code": "CA",
    "region_geoname_id": 5332921,
    "postal_code": "95141",
    "country": "United States",
    "country_code": "US",
    "country_geoname_id": 6252001,
    "country_is_eu": false,
    "continent": "North America",
    "continent_code": "NA",
    "continent_geoname_id": 6255149,
    "longitude": -121.7714,
    "latitude": 37.1835,
    "security": {
        "is_vpn": false
    },
    "timezone": {
        "name": "America/Los_Angeles",
        "abbreviation": "PDT",
        "gmt_offset": -7,
        "current_time": "06:37:41",
        "is_dst": true
    },
    "flag": {
        "emoji": "🇺🇸",
        "unicode": "U+1F1FA U+1F1F8",
        "png": "https://static.abstractapi.com/country-flags/US_flag.png",
        "svg": "https://static.abstractapi.com/country-flags/US_flag.svg"
    },
    "currency": {
        "currency_name": "USD",
        "currency_code": "USD"
    },
    "connection": {
        "autonomous_system_number": 20057,
        "autonomous_system_organization": "ATT-MOBILITY-LLC-AS20057",
        "connection_type": "Cellular",
        "isp_name": "AT&T Mobility LLC",
        "organization_name": "Service Provider Corporation"
    }
}

"#;

    #[test]
    #[ignore]
    fn test_request() {
        use std::env;
        let key = env::var("ABSTRACT_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(AbstractApi::new(key));
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("AbstractApi: {:#?}", result);

        let response = AbstractApiResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = AbstractApiResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip_address, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(lookup.ip, "1.1.1.1", "IP address not matching");
    }
}
