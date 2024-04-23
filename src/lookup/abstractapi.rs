//! <https://abstractapi.com> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// https://docs.abstractapi.com/ip-geolocation
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

impl ProviderResponse<AbstractApiResponse> for AbstractApiResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip_address
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::AbstractApi,
        );
        response.country = self.country;
        response.continent = self.continent;
        response.country_code = self.country_code;
        response.region = self.region;
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
            response.is_proxy = security.is_vpn;
        }

        response
    }
}

/// AbstractApi provider
pub struct AbstractApi;

impl Provider for AbstractApi {
    #[inline]
    fn get_endpoint(&self, key: &Option<String>, target: &Option<IpAddr>) -> String {
        let key = match key {
            Some(k) => format!("?api_key={}", k),
            None => "".to_string(),
        };
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("&ip_address={}", t),
            None => "".to_string(),
        };
        format!("https://ipgeolocation.abstractapi.com/v1/{}{}", key, target)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = AbstractApiResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::AbstractApi
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

    #[tokio::test]
    #[ignore]
    async fn test_request_target() {
        use std::env;
        let key = env::var("ABSTRACT_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(AbstractApi);
        let target = "8.8.8.8".parse().ok();
        let result = service.get_client(key, target).send().await;
        let result = super::super::handle_response(result).await.unwrap();
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
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
