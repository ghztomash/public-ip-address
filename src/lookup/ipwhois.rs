//! <https://ipwhois.io> lookup provider

use super::{ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://ipwhois.io/documentation>
#[derive(Serialize, Deserialize, Debug)]
pub struct IpWhoIsResponse {
    ip: String,
    continent: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    is_eu: Option<bool>,
    postal: Option<String>,
    connection: Option<Connection>,
    timezone: Option<Timezone>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Connection {
    asn: Option<i64>,
    org: Option<String>,
    isp: Option<String>,
    domain: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timezone {
    id: Option<String>,
}

impl ProviderResponse<IpWhoIsResponse> for IpWhoIsResponse {
    fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpWhoIs,
        );
        response.continent = self.continent;
        response.region = self.region;
        response.country = self.country;
        response.country_code = self.country_code;
        response.postal_code = self.postal;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        if let Some(timezone) = self.timezone {
            response.time_zone = timezone.id;
        }
        if let Some(connection) = self.connection {
            response.asn_org = connection.org;
            if let Some(asn) = connection.asn {
                response.asn = Some(format!("{asn}"));
            }
        }
        response
    }
}

/// IpWhoIs provider
pub struct IpWhoIs;

impl Provider for IpWhoIs {
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = target.map(|t| t.to_string()).unwrap_or_default();
        format!("https://ipwho.is/{}", target)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpWhoIsResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpWhoIs
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
  "success": true,
  "type": "IPv4",
  "continent": "North America",
  "continent_code": "NA",
  "country": "United States",
  "country_code": "US",
  "region": "California",
  "region_code": "CA",
  "city": "Mountain View",
  "latitude": 37.3860517,
  "longitude": -122.0838511,
  "is_eu": false,
  "postal": "94039",
  "calling_code": "1",
  "capital": "Washington D.C.",
  "borders": "CA,MX",
  "flag": {
    "img": "https://cdn.ipwhois.io/flags/us.svg",
    "emoji": "🇺🇸",
    "emoji_unicode": "U+1F1FA U+1F1F8"
  },
  "connection": {
    "asn": 15169,
    "org": "Google LLC",
    "isp": "Google LLC",
    "domain": "google.com"
  },
  "timezone": {
    "id": "America/Los_Angeles",
    "abbr": "PDT",
    "is_dst": true,
    "offset": -25200,
    "utc": "-07:00",
    "current_time": "2024-03-21T16:47:26-07:00"
  }
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(IpWhoIs);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpWhoIs: {:#?}", result);
        let response = IpWhoIsResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpWhoIsResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
