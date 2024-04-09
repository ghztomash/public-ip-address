//! <https://getjsonip.com> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://getjsonip.com
#[derive(Serialize, Deserialize, Debug)]
pub struct GetJsonIpResponse {
    ip: String,
}

impl GetJsonIpResponse {
    pub fn parse(input: String) -> Result<GetJsonIpResponse> {
        let deserialized: GetJsonIpResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::GetJsonIp,
        )
    }
}

pub struct GetJsonIp;
impl Provider for GetJsonIp {
    fn make_api_request(&self, _key: Option<String>, _target: Option<IpAddr>) -> Result<String> {
        let response = reqwest::blocking::get("https://ipv4.jsonip.com");
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = GetJsonIpResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::GetJsonIp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "ip": "1.1.1.1"
}
"#;

    #[test]
    fn test_request() {
        let service = Box::new(GetJsonIp);
        let result = service.make_api_request(None, None);
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("GetJsonIp: {:#?}", result);
        let response = GetJsonIpResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = GetJsonIpResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
