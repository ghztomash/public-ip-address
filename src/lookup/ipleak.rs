//! <https://ipleak.net> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// https://ipleak.net/
#[derive(Serialize, Deserialize, Debug)]
pub struct IpLeakResponse {
    ip: String,
    city_name: Option<String>,
    region_name: Option<String>,
    region_code: Option<String>,
    country_name: Option<String>,
    country_code: Option<String>,
    continent_name: Option<String>,
    continent_code: Option<String>,
    postal_code: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    time_zone: Option<String>,
    isp_name: Option<String>,
    as_number: Option<i64>,
    reverse: Option<String>,
}

impl ProviderResponse<IpLeakResponse> for IpLeakResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpLeak,
        );
        response.country = self.country_name;
        response.country_code = self.country_code;
        response.region = self.region_name;
        response.postal_code = self.postal_code;
        response.continent = self.continent_name;
        response.city = self.city_name;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        response.asn_org = self.isp_name;
        if let Some(asn) = self.as_number {
            response.asn = Some(asn.to_string());
        }
        response.hostname = self.reverse;
        response
    }
}

/// IpLeak lookup provider
pub struct IpLeak;

impl Provider for IpLeak {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => t,
            None => "".to_string(),
        };
        format!("https://ipleak.net/json/{}", target)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpLeakResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpLeak
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
    "as_number": 15169,
    "isp_name": "GOOGLE",
    "country_code": "US",
    "country_name": "United States",
    "region_code": null,
    "region_name": null,
    "continent_code": "NA",
    "continent_name": "North America",
    "city_name": null,
    "postal_code": null,
    "postal_confidence": null,
    "latitude": 37.751,
    "longitude": -97.822,
    "accuracy_radius": 1000,
    "time_zone": "America\/Chicago",
    "metro_code": null,
    "level": "min",
    "cache": 1712158151,
    "ip": "8.8.8.8",
    "reverse": "",
    "query_text": "8.8.8.8",
    "query_type": "ip",
    "query_date": 1712158151
}
"#;

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        let service = Box::new(IpLeak);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpLeak: {:#?}", result);
        let response = IpLeakResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpLeakResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "8.8.8.8", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "8.8.8.8".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
