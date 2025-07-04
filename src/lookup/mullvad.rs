//! <https://mullvad.net> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider, ProviderResponse},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://mullvad.net/>
#[derive(Serialize, Deserialize, Debug)]
pub struct MullvadResponse {
    ip: String,
    city: Option<String>,
    country: Option<String>,
    organization: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    mullvad_exit_ip: Option<bool>,
}

impl ProviderResponse<MullvadResponse> for MullvadResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::Mullvad,
        );
        response.country = self.country;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.asn_org = self.organization;
        response.is_proxy = self.mullvad_exit_ip;
        response
    }
}

/// Mullvad lookup provider
pub struct Mullvad;

impl Provider for Mullvad {
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        "https://am.i.mullvad.net/json".to_string()
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = MullvadResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Mullvad
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
    "organization": "GOOGLE",
    "country": "United States",
    "city": "New York",
    "latitude": 37.751,
    "longitude": -97.822,
    "mullvad_exit_ip": false,
    "blacklisted": {
      "blacklisted": false,
      "results": []
    },
    "ip": "8.8.8.8"
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(Mullvad);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Mullvad: {result:#?}");
        let response = MullvadResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[test]
    fn test_parse() {
        let response = MullvadResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "8.8.8.8", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "8.8.8.8".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
