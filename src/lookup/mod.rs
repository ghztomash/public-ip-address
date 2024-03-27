use crate::{error::LookupError, LookupResponse};
use reqwest::{blocking::Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod freeipapi;
pub mod ifconfig;
pub mod ipapicom;
pub mod ipapico;
pub mod ipinfo;
pub mod ipwhois;
pub mod mock;
pub mod myip;

/// Result type for the lookup crate
pub type Result<T> = std::result::Result<T, LookupError>;

pub trait Provider {
    fn make_api_request(&self) -> Result<String>;
    fn parse_reply(&self, json: String) -> Result<LookupResponse>;
    fn get_type(&self) -> LookupProvider;
}

pub struct LookupService {
    provider: Box<dyn Provider>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum LookupProvider {
    FreeIpApi,
    IfConfig,
    IpInfo,
    MyIp,
    IpApiCom,
    IpWhoIs,
    IpApiCo,
    Mock(String),
}

impl fmt::Display for LookupProvider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl LookupProvider {
    fn build(&self) -> Box<dyn Provider> {
        match self {
            LookupProvider::FreeIpApi => Box::new(freeipapi::FreeIpApi),
            LookupProvider::IfConfig => Box::new(ifconfig::IfConfig),
            LookupProvider::IpInfo => Box::new(ipinfo::IpInfo),
            LookupProvider::MyIp => Box::new(myip::MyIp),
            LookupProvider::IpApiCom => Box::new(ipapicom::IpApiCom),
            LookupProvider::IpApiCo => Box::new(ipapico::IpApiCo),
            LookupProvider::IpWhoIs => Box::new(ipwhois::IpWhoIs),
            LookupProvider::Mock(ref ip) => Box::new(mock::Mock { ip: ip.to_string() }),
        }
    }
}

impl LookupService {
    pub fn new(provider: LookupProvider) -> Self {
        LookupService {
            provider: provider.build(),
        }
    }

    pub fn set_provider(&mut self, provider: LookupProvider) -> &Self {
        self.provider = provider.build();
        self
    }

    pub fn get_provider_type(&self) -> LookupProvider {
        self.provider.get_type()
    }

    pub fn make_request(&self) -> Result<LookupResponse> {
        let response = self.provider.make_api_request()?;
        self.provider.parse_reply(response)
    }
}

fn handle_response(response: reqwest::Result<Response>) -> Result<String> {
    match response {
        Ok(response) => match response.status() {
            StatusCode::OK => Ok(response.text()?),
            StatusCode::TOO_MANY_REQUESTS => Err(LookupError::TooManyRequests(format!(
                "Too many requests: {}",
                response.status()
            ))),
            s => Err(LookupError::RequestStatus(format!("Status: {}", s))),
        },
        Err(e) => Err(LookupError::ReqwestError(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_provider() {
        let mut provider = LookupService::new(LookupProvider::IpApi);
        assert_eq!(provider.get_provider_type(), LookupProvider::IpApi);
        provider.set_provider(LookupProvider::IpInfo);
        assert_eq!(provider.get_provider_type(), LookupProvider::IpInfo);
    }

    #[test]
    fn test_make_request() {
        let address = "1.1.1.1".to_string();
        let provider = LookupService::new(LookupProvider::Mock(address.to_string()));
        let response = provider.make_request().unwrap();
        assert_eq!(response.ip, address);
    }

    #[test]
    fn test_handle_response() {
        let response = reqwest::blocking::get("https://httpbin.org/status/200");
        let body = handle_response(response);
        assert!(body.is_ok(), "Response is an error {:#?}", body);
    }

    #[test]
    fn test_handle_response_error() {
        let response = reqwest::blocking::get("https://httpbin.org/status/500");
        let body = handle_response(response);
        assert!(body.is_err(), "Response should be an error {:#?}", body);
        let body = body.unwrap_err();
        assert_eq!(
            body.to_string(),
            "Request status",
            "Wrong error {:#?}",
            body
        );
    }

    #[test]
    fn test_handle_response_too_many() {
        let response = reqwest::blocking::get("https://httpbin.org/status/429");
        let body = handle_response(response);
        assert!(body.is_err(), "Response should be an error {:#?}", body);
        let body = body.unwrap_err();
        assert_eq!(
            body.to_string(),
            "Too many API requests",
            "Wrong error {:#?}",
            body
        );
    }
}
