//! <https://ipgeolocation.io> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};

// https://ipgeolocation.io/documentation
#[derive(Serialize, Deserialize, Debug)]
pub struct IpGeolocationResponse {
    ip: String,
    hostname: Option<String>,
    continent_code: Option<String>,
    continent_name: Option<String>,
    country_code2: Option<String>,
    country_code3: Option<String>,
    country_name: Option<String>,
    state_prov: Option<String>,
    state_code: Option<String>,
    city: Option<String>,
    zipcode: Option<String>,
    longitude: Option<String>,
    latitude: Option<String>,
    is_eu: Option<bool>,
    connection_type: Option<String>,
    organization: Option<String>,
    isp: Option<String>,
    time_zone: Option<Timezone>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timezone {
    name: Option<String>,
}

impl IpGeolocationResponse {
    pub fn parse(input: String) -> Result<IpGeolocationResponse> {
        let deserialized: IpGeolocationResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(self.ip, LookupProvider::IpGeolocation(None));
        response.continent = self.continent_name;
        response.country = self.country_name;
        response.country_code = self.country_code2;
        response.region = self.state_prov;
        response.region_code = self.state_code;
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

pub struct IpGeolocation {
    key: Option<String>,
}

impl IpGeolocation {
    /// Create a new IpGeolocation instance with an API key
    pub fn new(key: Option<String>) -> IpGeolocation {
        IpGeolocation { key }
    }
}

impl Provider for IpGeolocation {
    fn make_api_request(&self) -> Result<String> {
        let endpoint = format!(
            "https://api.ipgeolocation.io/ipgeo?apiKey={}",
            self.key.as_ref().unwrap_or(&"".to_string())
        );
        let response = reqwest::blocking::get(endpoint);
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpGeolocationResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpGeolocation(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
    "ip": "8.8.8.8",
    "hostname": "dns.google",
    "continent_code": "NA",
    "continent_name": "North America",
    "country_code2": "US",
    "country_code3": "USA",
    "country_name": "United States",
    "country_capital": "Washington, D.C.",
    "state_prov": "California",
    "district": "Santa Clara",
    "city": "Mountain View",
    "zipcode": "94043-1351",
    "latitude": "37.42240",
    "longitude": "-122.08421",
    "is_eu": false,
    "calling_code": "+1",
    "country_tld": ".us",
    "languages": "en-US,es-US,haw,fr",
    "country_flag": "https://ipgeolocation.io/static/flags/us_64.png",
    "geoname_id": "6301403",
    "isp": "Google LLC",
    "connection_type": "",
    "organization": "Google LLC",
    "asn": "AS15169",
    "currency": {
        "code": "USD",
        "name": "US Dollar",
        "symbol": "$"
    },
    "time_zone": {
        "name": "America/Los_Angeles",
        "offset": -8,
        "current_time": "2020-12-17 07:49:45.872-0800",
        "current_time_unix": 1608220185.872,
        "is_dst": false,
        "dst_savings": 1
    }
}
"#;

    #[test]
    #[ignore]
    fn test_request() {
        use std::env;
        let key = env::var("IPGEOLOCATION_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(IpGeolocation::new(key));
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpGeolocation: {:#?}", result);

        let response = IpGeolocationResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpGeolocationResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "8.8.8.8", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(lookup.ip, "8.8.8.8", "IP address not matching");
    }
}
