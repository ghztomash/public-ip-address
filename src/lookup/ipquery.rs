//! <https://ipquery.io/> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://ipquery.gitbook.io/ipquery-docs>
#[derive(Serialize, Deserialize, Debug)]
pub struct IpQueryResponse {
    ip: String,
    isp: Option<Isp>,
    location: Option<Location>,
    risk: Option<Risk>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Isp {
    asn: Option<String>,
    org: Option<String>,
    isp: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    country: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zipcode: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<String>,
    localtime: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Risk {
    is_mobile: Option<bool>,
    is_vpn: Option<bool>,
    is_tor: Option<bool>,
    is_proxy: Option<bool>,
    is_datacenter: Option<bool>,
    risk_score: Option<i64>,
}

impl ProviderResponse<IpQueryResponse> for IpQueryResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpQuery,
        );
        if let Some(location) = self.location {
            response.country = location.country;
            response.country_code = location.country_code;
            response.city = location.city;
            response.postal_code = location.zipcode;
            response.latitude = location.latitude;
            response.longitude = location.longitude;
            response.time_zone = location.timezone;
            response.region = location.state;
        }
        if let Some(isp) = self.isp {
            response.asn_org = isp.org;
            response.asn = isp.asn;
        }
        if let Some(risk) = self.risk {
            let is_proxy = risk.is_proxy.unwrap_or(false)
                || risk.is_vpn.unwrap_or(false)
                || risk.is_tor.unwrap_or(false);
            response.is_proxy = Some(is_proxy);
        }
        response
    }
}

/// IpQuery provider
pub struct IpQuery;

impl Provider for IpQuery {
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = target.map(|t| t.to_string()).unwrap_or_default();
        format!("https://api.ipquery.io/{target}?format=json")
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpQueryResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpQuery
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
    "isp": {
        "asn": "AS13335",
        "org": "Cloudflare, Inc.",
        "isp": "Cloudflare, Inc."
    },
    "location": {
        "country": "Australia",
        "country_code": "AU",
        "city": "Sydney",
        "state": "New South Wales",
        "zipcode": "1001",
        "latitude": -33.854548400186665,
        "longitude": 151.20016200912815,
        "timezone": "Australia/Sydney",
        "localtime": "2024-09-03T22:22:52"
    },
    "risk": {
        "is_mobile": false,
        "is_vpn": false,
        "is_tor": false,
        "is_proxy": false,
        "is_datacenter": true,
        "risk_score": 0
    }
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(IpQuery);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpQuery: {result:#?}");
        let response = IpQueryResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[test]
    fn test_parse() {
        let response = IpQueryResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
