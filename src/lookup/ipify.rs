//! <https://ipify.org> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://www.ipify.org
#[derive(Serialize, Deserialize, Debug)]
pub struct IpifyResponse {
    ip: String,
}

impl IpifyResponse {
    pub fn parse(input: String) -> Result<IpifyResponse> {
        let deserialized: IpifyResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::Ipify,
        )
    }
}

pub struct Ipify;

#[async_trait::async_trait]
impl Provider for Ipify {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        "https://api64.ipify.org/?format=json".to_string()
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpifyResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Ipify
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

    #[tokio::test]
    async fn test_request() {
        let service = Box::new(Ipify);
        let result = service.make_api_request(None, None).await;
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("Ipify: {:#?}", result);
        let response = IpifyResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpifyResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
