//! <https://iplocate.io> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://www.iplocate.io/docs/>
#[derive(Serialize, Deserialize, Debug)]
pub struct IpLocateIoResponse {
    ip: String,
    country: Option<String>,
    country_code: Option<String>,
    is_eu: Option<bool>,
    city: Option<String>,
    continent: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    time_zone: Option<String>,
    postal_code: Option<String>,
    subdivision: Option<String>,
    network: Option<String>,
    asn: Option<Asn>,
    company: Option<Company>,
    privacy: Option<Privacy>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Asn {
    asn: Option<String>,
    route: Option<String>,
    netname: Option<String>,
    name: Option<String>,
    country_code: Option<String>,
    domain: Option<String>,
    #[serde(rename = "type")]
    asn_type: Option<String>,
    rir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Company {
    name: Option<String>,
    domain: Option<String>,
    country_code: Option<String>,
    #[serde(rename = "type")]
    company_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Privacy {
    is_abuser: Option<bool>,
    is_anonymous: Option<bool>,
    is_bogon: Option<bool>,
    is_icoud_relay: Option<bool>,
    is_vpn: Option<bool>,
    is_tor: Option<bool>,
    is_proxy: Option<bool>,
    is_datacenter: Option<bool>,
}

impl ProviderResponse<IpLocateIoResponse> for IpLocateIoResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpLocateIo,
        );
        response.country = self.country;
        response.continent = self.continent;
        response.country_code = self.country_code;
        response.region = self.subdivision;
        response.postal_code = self.postal_code;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone;
        if let Some(asn) = self.asn {
            response.asn_org = asn.name;
            response.asn = asn.asn;
        }
        if let Some(privacy) = self.privacy {
            let is_proxy = privacy.is_proxy.unwrap_or(false)
                || privacy.is_vpn.unwrap_or(false)
                || privacy.is_tor.unwrap_or(false);
            response.is_proxy = Some(is_proxy);
        }
        response
    }
}

/// IpLocateIo lookup provider
pub struct IpLocateIo;

impl Provider for IpLocateIo {
    fn get_endpoint(&self, key: &Option<String>, target: &Option<IpAddr>) -> String {
        let key = match key {
            Some(k) => format!("?apikey={k}"),
            None => "".to_string(),
        };
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("/{t}"),
            None => "".to_string(),
        };
        format!("https://iplocate.io/api/lookup{target}/json{key}")
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpLocateIoResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpLocateIo
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
  "country": "Australia",
  "country_code": "AU",
  "is_eu": false,
  "city": "Sydney",
  "continent": "Oceania",
  "latitude": -33.8672,
  "longitude": 151.1997,
  "time_zone": "Australia/Sydney",
  "postal_code": "2049",
  "subdivision": "New South Wales",
  "subdivision2": null,
  "network": "123.243.240.0/20",
  "asn": {
    "asn": "AS7545",
    "route": "123.243.246.0/24",
    "netname": "TPG-INTERNET-AP",
    "name": "TPG Telecom Limited",
    "country_code": "AU",
    "domain": "tpgtelecom.com.au",
    "type": "isp",
    "rir": "APNIC"
  },
  "privacy": {
    "is_abuser": false,
    "is_anonymous": false,
    "is_bogon": false,
    "is_datacenter": false,
    "is_icloud_relay": false,
    "is_proxy": false,
    "is_tor": false,
    "is_vpn": false
  },
  "company": {
    "name": "TPG Telecom",
    "domain": "www.tpgtelecom.com.au",
    "country_code": "AU",
    "type": "isp"
  },
  "abuse": {
    "address": "TPG Internet Pty Ltd., (Part of the Total Peripherals Group), 65 Waterloo Road, North Ryde NSW 2113",
    "email": "hostmaster@tpgtelecom.com.au",
    "name": "ABUSE TPGCOMAU",
    "network": "123.243.246.192 - 123.243.246.223",
    "phone": "+000000000"
  }
}"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(IpLocateIo);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpLocateIo: {result:#?}");
        let response = IpLocateIoResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request_target() {
        let service = Box::new(IpLocateIo);
        let result = service.get_client(None, Some("8.8.8.8".parse().unwrap())).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpLocateIo: {result:#?}");
        let response = IpLocateIoResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
        assert_eq!(response.unwrap().ip, "8.8.8.8");
    }

    #[test]
    fn test_parse() {
        let response = IpLocateIoResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
