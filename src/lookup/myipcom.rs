//! <https://myip.com> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://www.myip.com/api-docs
#[derive(Serialize, Deserialize, Debug)]
pub struct MyIpComResponse {
    ip: String,
    country: Option<String>,
    cc: Option<String>,
}

impl MyIpComResponse {
    pub fn parse(input: String) -> Result<MyIpComResponse> {
        let deserialized: MyIpComResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::MyIpCom,
        );
        response.country = self.country;
        response.country_code = self.cc;

        response
    }
}

pub struct MyIpCom;

#[async_trait::async_trait]
impl Provider for MyIpCom {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        "https://api.myip.com".to_string()
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = MyIpComResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::MyIpCom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "ip": "1.1.1.1",
  "cc": "DE",
  "country": "Germany"
}
"#;

    #[tokio::test]
    async fn test_request() {
        let service = Box::new(MyIpCom);
        let result = service.make_api_request(None, None).await;
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("MyIpCom: {:#?}", result);
        let response = MyIpComResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = MyIpComResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
