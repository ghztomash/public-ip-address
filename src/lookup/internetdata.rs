//! <https://internetdata.io> lookup provider

use super::{client::RequestBuilder, ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://docs.internetdata.io/api>
#[derive(Serialize, Deserialize, Debug)]
pub struct InternetDataResponse {
    ip: String,
    is_vpn: Option<bool>,
    is_tor: Option<bool>,
    is_resproxy: Option<bool>,
    is_dcproxy: Option<bool>,
    is_mobproxy: Option<bool>,
    is_pubproxy: Option<bool>,
    location: Option<Location>,
    asn: Option<Asn>,
    hostname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    continent_name: Option<String>,
    country: Option<String>,
    country_name: Option<String>,
    region: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Asn {
    asn: Option<u32>,
    name: Option<String>,
}

/// Turns an empty string into `None`
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

impl ProviderResponse<InternetDataResponse> for InternetDataResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::InternetData,
        );
        if let Some(location) = self.location {
            response.continent = non_empty(location.continent_name);
            response.country = non_empty(location.country_name);
            response.country_code = non_empty(location.country);
            response.region = non_empty(location.region);
            response.city = non_empty(location.city);
            response.postal_code = non_empty(location.postal_code);
            response.latitude = location.latitude;
            response.longitude = location.longitude;
            response.time_zone = non_empty(location.timezone);
        }
        if let Some(asn) = self.asn {
            response.asn = asn.asn.filter(|a| *a != 0).map(|a| format!("AS{a}"));
            response.asn_org = non_empty(asn.name);
        }
        response.hostname = non_empty(self.hostname);
        // VPN and proxy flags come with an API key; without one only Tor is known
        if self.is_vpn.is_some() {
            let is_proxy = self.is_vpn.unwrap_or(false)
                || self.is_tor.unwrap_or(false)
                || self.is_resproxy.unwrap_or(false)
                || self.is_dcproxy.unwrap_or(false)
                || self.is_mobproxy.unwrap_or(false)
                || self.is_pubproxy.unwrap_or(false);
            response.is_proxy = Some(is_proxy);
        } else if self.is_tor == Some(true) {
            response.is_proxy = Some(true);
        }
        response
    }
}

/// InternetData lookup provider
pub struct InternetData;

impl Provider for InternetData {
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => t,
            None => "myip".to_string(),
        };
        format!("https://api.internetdata.io/{target}")
    }

    fn add_auth(&self, request: RequestBuilder, key: &Option<String>) -> RequestBuilder {
        if let Some(key) = key {
            return request.bearer_auth(key);
        }
        request
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = InternetDataResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::InternetData
    }

    fn supports_target_lookup(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "ip": "1.1.1.1",
  "is_bogon": false,
  "is_tor": false,
  "is_relay": false,
  "is_anycast": true,
  "is_carrier": false,
  "is_satellite": false,
  "is_ssh": false,
  "is_smtp": false,
  "is_web": true,
  "is_dns_resolver": true,
  "is_bittorrent": false,
  "location": {
    "continent": "OC",
    "continent_name": "Oceania",
    "country": "AU",
    "country_name": "Australia",
    "timezone": "Australia/Brisbane",
    "currency_code": "AUD",
    "currency_name": "Dollar",
    "currency_symbol": "A$",
    "flag_emoji": "🇦🇺",
    "flag_unicode": "U+1F1E6 U+1F1FA"
  },
  "tor": {},
  "relay": {},
  "asn": {
    "asn": 13335,
    "name": "Cloudflare, Inc.",
    "domain": "cloudflare.com",
    "type": "hosting",
    "prefix": "1.1.1.0/24",
    "country": "US",
    "rpki_status": "valid"
  },
  "company": {
    "name": "APNIC and Cloudflare DNS Resolver project",
    "domain": "cloudflare.com"
  },
  "hostname": "one.one.one.one",
  "abuse": {
    "email": "abuse@cloudflare.com"
  }
}"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(InternetData);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("InternetData: {result:#?}");
        let response = InternetDataResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request_target() {
        let service = Box::new(InternetData);
        let result = service
            .get_client(None, Some("8.8.8.8".parse().unwrap()))
            .send()
            .await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("InternetData: {result:#?}");
        let response = InternetDataResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
        assert_eq!(response.unwrap().ip, "8.8.8.8");
    }

    #[test]
    fn test_parse() {
        let response = InternetDataResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
        assert_eq!(lookup.country_code, Some("AU".to_string()));
        assert_eq!(lookup.asn, Some("AS13335".to_string()));
        assert_eq!(lookup.city, None, "City needs an API key");
        assert_eq!(lookup.is_proxy, None, "Proxy flags need an API key");
    }
}
