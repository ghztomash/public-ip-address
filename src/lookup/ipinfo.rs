//! <https://ipinfo.io> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://ipinfo.io/json>
#[derive(Serialize, Deserialize, Debug)]
pub struct IpInfoResponse {
    ip: String,
    hostname: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    loc: Option<String>,
    org: Option<String>,
    postal: Option<String>,
    timezone: Option<String>,
    readme: Option<String>,
}

impl ProviderResponse<IpInfoResponse> for IpInfoResponse {
    fn into_response(self) -> LookupResponse {
        let mut latitude = None;
        let mut longitude = None;

        // convert loc string to float
        if let Some(loc) = self.loc {
            let coords: Vec<&str> = loc.split(',').collect();
            if coords.len() == 2 {
                latitude = coords[0].parse().ok();
                longitude = coords[1].parse().ok();
            }
        }

        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpInfo,
        );
        response.country = self.country.clone();
        response.country_code = self.country;
        response.region = self.region;
        response.postal_code = self.postal;
        response.city = self.city;
        response.latitude = latitude;
        response.longitude = longitude;
        response.time_zone = self.timezone;
        response.asn_org = self.org.clone();
        response.asn = self.org;
        response
    }
}

/// IpInfo lookup provider
pub struct IpInfo;

impl Provider for IpInfo {
    #[inline]
    fn get_endpoint(&self, key: &Option<String>, target: &Option<IpAddr>) -> String {
        let key = match key {
            Some(k) => format!("?token={}", k),
            None => "".to_string(),
        };
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("{}/", t),
            None => "".to_string(),
        };
        format!("https://ipinfo.io/{}json{}", target, key)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpInfoResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpInfo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "ip": "1.1.1.1",
  "hostname": "ip-66-87-125-72.spfdma.spcsdns.net",
  "city": "Springfield",
  "region": "Massachusetts",
  "country": "US",
  "loc": "42.1015,-72.5898",
  "org": "AS10507 Sprint Personal Communications Systems",
  "postal": "01101",
  "timezone": "America/New_York"
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(IpInfo);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpInfo: {:#?}", result);
        let response = IpInfoResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpInfoResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
