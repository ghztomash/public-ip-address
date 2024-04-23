//! <https://ipgeolocation.io> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// https://ipgeolocation.io/documentation
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

impl ProviderResponse<IpGeolocationResponse> for IpGeolocationResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpGeolocation,
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

/// IpGeolocation lookup provider
pub struct IpGeolocation;

impl Provider for IpGeolocation {
    #[inline]
    fn get_endpoint(&self, key: &Option<String>, target: &Option<IpAddr>) -> String {
        let key = match key {
            Some(k) => format!("?apiKey={}", k),
            None => "".to_string(),
        };
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("&ip={}", t),
            None => "".to_string(),
        };
        format!("https://api.ipgeolocation.io/ipgeo{}{}", key, target)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpGeolocationResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpGeolocation
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

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        use std::env;
        let key = env::var("IPGEOLOCATION_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(IpGeolocation);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpGeolocation: {:#?}", result);

        let response = IpGeolocationResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[tokio::test]
    #[ignore]
    async fn test_request_target() {
        use std::env;
        let key = env::var("IPGEOLOCATION_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let target = "8.8.8.8".parse().ok();
        let service = Box::new(IpGeolocation);
        let result = service.get_client(key, target).send().await;
        let result = super::super::handle_response(result).await.unwrap();
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
        assert_eq!(
            lookup.ip,
            "8.8.8.8".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
