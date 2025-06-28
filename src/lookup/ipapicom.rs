//! <https://ip-api.com> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// <http://ip-api.com/docs/api:json>
pub struct IpApiComResponse {
    query: String,
    status: Option<String>,
    continent: Option<String>,
    continent_code: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    region: Option<String>,
    region_name: Option<String>,
    city: Option<String>,
    district: Option<String>,
    zip: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<String>,
    offset: Option<i64>,
    currency: Option<String>,
    isp: Option<String>,
    org: Option<String>,
    #[serde(rename = "as")]
    asn: Option<String>,
    as_name: Option<String>,
    reverse: Option<String>,
    mobile: Option<bool>,
    proxy: Option<bool>,
    hosting: Option<bool>,
}

impl ProviderResponse<IpApiComResponse> for IpApiComResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.query
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpApiCom,
        );
        response.country = self.country;
        response.country_code = self.country_code;
        response.region = self.region_name;
        response.postal_code = self.zip;
        response.city = self.city;
        response.latitude = self.lat;
        response.longitude = self.lon;
        response.time_zone = self.timezone;
        response.asn_org = self.org;
        response.asn = self.asn;
        response.hostname = self.reverse;
        response.is_proxy = self.proxy;
        response
    }
}

/// IpApiCom lookup provider
pub struct IpApiCom;

impl Provider for IpApiCom {
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => t,
            None => "".to_string(),
        };
        format!("http://ip-api.com/json/{target}?fields=66846719")
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpApiComResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpApiCom
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
  "query": "1.1.1.1",
  "status": "success",
  "continent": "North America",
  "continentCode": "NA",
  "country": "Canada",
  "countryCode": "CA",
  "region": "QC",
  "regionName": "Quebec",
  "city": "Montreal",
  "district": "",
  "zip": "H1K",
  "lat": 45.6085,
  "lon": -73.5493,
  "timezone": "America/Toronto",
  "offset": -14400,
  "currency": "CAD",
  "isp": "Le Groupe Videotron Ltee",
  "org": "Videotron Ltee",
  "as": "AS5769 Videotron Ltee",
  "asname": "VIDEOTRON",
  "reverse": "modemcable001.0-48-24.mc.videotron.ca",
  "mobile": false,
  "proxy": false,
  "hosting": false
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(IpApiCom);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpApiCom: {result:#?}");
        let response = IpApiComResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[test]
    fn test_parse() {
        let response = IpApiComResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.query, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
