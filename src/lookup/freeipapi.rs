//! <https://freeipapi.com> lookup provider

use super::{client::RequestBuilder, ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// https://docs.freeipapi.com/response.html
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FreeIpApiResponse {
    ip_version: u8,
    ip_address: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    country_name: Option<String>,
    country_code: Option<String>,
    time_zone: Option<String>,
    zip_code: Option<String>,
    city_name: Option<String>,
    region_name: Option<String>,
    continent: Option<String>,
    continent_code: Option<String>,
    is_proxy: Option<bool>,
}

impl ProviderResponse<FreeIpApiResponse> for FreeIpApiResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip_address
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::FreeIpApi,
        );
        response.country = self.country_name;
        response.country_code = self.country_code;
        response.continent = self.continent;
        response.region = self.region_name;
        response.postal_code = self.zip_code;
        response.city = self.city_name;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        response.is_proxy = self.is_proxy;
        response
    }
}

/// FreeIpApi lookup provider
pub struct FreeIpApi;

impl Provider for FreeIpApi {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => t,
            None => "".to_string(),
        };
        format!("https://freeipapi.com/api/json/{}", target)
    }

    #[inline]
    fn add_auth(&self, request: RequestBuilder, key: &Option<String>) -> RequestBuilder {
        if let Some(key) = key {
            return request.bearer_auth(key);
        }
        request
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = FreeIpApiResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::FreeIpApi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
 "ipVersion": 4,
 "ipAddress": "1.1.1.1",
 "latitude": 58.416588,
 "longitude": 15.616713,
 "countryName": "Sweden",
 "countryCode": "SE",
 "timeZone": "+02:00",
 "zipCode": "58957",
 "cityName": "Linkoping",
 "regionName": "Ostergotlands lan",
 "continent": "Europe",
 "continentCode": "EU"
}
"#;

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        let service = Box::new(FreeIpApi);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("FreeIpApi: {:#?}", result);
        let response = FreeIpApiResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = FreeIpApiResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip_address, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
