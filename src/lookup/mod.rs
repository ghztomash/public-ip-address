//! # 🛠️ Lookup service provider module
//!
//! The `lookup` module provides functionality for performing public IP lookups from various services.
//! It includes a `LookupService` struct for making requests to a lookup provider, and a `LookupProvider` enum for specifying the provider.
//!
//! ## Example
//! ```rust
//! use public_ip_address::lookup::{LookupProvider, LookupService};
//! use std::{error::Error, str::FromStr, net::IpAddr};
//!
//! #[cfg_attr(not(feature = "blocking"), tokio::main)]
//! #[maybe_async::maybe_async]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let provider = LookupProvider::from_str("ipinfo")?;
//!     let service = LookupService::new(provider, None);
//!     let target = "8.8.8.8".parse::<IpAddr>().ok();
//!     let result = service.lookup(target).await?;
//!     println!("{}", result);
//!     Ok(())
//! }
//! ```

use crate::LookupResponse;
use client::{Client, RequestBuilder, Response};
use error::{LookupError, Result};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr, str::FromStr};

mod client;

pub mod abstractapi;
pub mod error;
pub mod freeipapi;
pub mod getjsonip;
pub mod ifconfig;
pub mod ip2location;
pub mod ipapico;
pub mod ipapicom;
pub mod ipapiio;
pub mod ipbase;
pub mod ipdata;
pub mod ipgeolocation;
pub mod ipify;
pub mod ipinfo;
pub mod ipleak;
pub mod iplocateio;
pub mod ipquery;
pub mod ipwhois;
pub mod mock;
pub mod mullvad;
pub mod myip;
pub mod myipcom;

/// Provider trait to define the methods that a provider must implement
pub trait Provider {
    /// Returns the API endpoint for the provider
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String;
    /// Parses the response from the provider
    fn parse_reply(&self, json: String) -> Result<LookupResponse>;
    /// Returns the type enum of the provider
    fn get_type(&self) -> LookupProvider;

    /// Returns a request client for the provider
    fn get_client(&self, key: Option<String>, target: Option<IpAddr>) -> RequestBuilder {
        let client = Client::new().get(self.get_endpoint(&key, &target));
        self.add_auth(client, &key)
    }

    /// Add authentication header to the request
    fn add_auth(&self, request: RequestBuilder, _key: &Option<String>) -> RequestBuilder {
        request
    }

    /// Check if the provider supports target lookup
    fn supports_target_lookup(&self) -> bool {
        false
    }

    /// Check if the provider supports bulk lookup
    fn supports_bulk_lookup(&self) -> bool {
        false
    }
}

/// ProviderResponse trait that define methods to parse the response from the provider
pub trait ProviderResponse<T: DeserializeOwned> {
    /// Parse the response json into a concrete type
    fn parse(input: String) -> Result<T> {
        let deserialized: T = serde_json::from_str(&input)?;
        Ok(deserialized)
    }
    /// Convert the response into a LookupResponse
    fn into_response(self) -> LookupResponse;
}

/// Available lookup service providers
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum LookupProvider {
    /// FreeIpApi provider (<https://freeipapi.com>)
    FreeIpApi,
    /// IfConfig provider (<https://ifconfig.co>)
    IfConfig,
    /// IpInfo provider (<https://ipinfo.io>)
    IpInfo,
    /// MyIp provider (<https://my-ip.io>)
    MyIp,
    /// IpApiCom provider (<https://ip-api.com>)
    IpApiCom,
    /// IpWhoIs provider (<https://ipwhois.io>)
    IpWhoIs,
    /// IpApiCo provider (<https://ipapi.co>)
    IpApiCo,
    /// IpApiIo provider (<https://ip-api.io>)
    IpApiIo,
    /// IpBase provider (<https://ipbase.com>)
    IpBase,
    /// IpLocateIo provider (<https://iplocate.io>)
    IpLocateIo,
    /// IpLeak provider (<https://ipleak.net>)
    IpLeak,
    /// Mullvad provider (<https://mullvad.net>)
    Mullvad,
    /// Abstract provider (<https://abstractapi.com>)
    AbstractApi,
    /// IpGeolocation provider (<https://ipgeolocation.io>)
    IpGeolocation,
    /// IpData provider (<https://ipdata.co>)
    IpData,
    /// Ip2Location provider (<https://www.ip2location.io>)
    Ip2Location,
    /// MyIpCom provider (<https://www.myip.com>)
    MyIpCom,
    /// Ipify provider (<https://www.ipify.org>)
    Ipify,
    /// GetJsonIp provider (<https://getjsonip.com>)
    GetJsonIp,
    /// IpQuery provider (<https://ipquery.io>)
    IpQuery,
    /// Mock provider for testing
    Mock(String),
}

impl fmt::Display for LookupProvider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for LookupProvider {
    type Err = LookupError;
    /// Parse a `&str` into a LookupProvider
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();
        // split the string into parts
        let s = s
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<String>>();
        // get the provider
        let p = s
            .first()
            .ok_or(LookupError::GenericError("No provider given".to_string()))?;

        match p.as_str() {
            "freeipapi" => Ok(LookupProvider::FreeIpApi),
            "ifconfig" => Ok(LookupProvider::IfConfig),
            "ipinfo" => Ok(LookupProvider::IpInfo),
            "myip" => Ok(LookupProvider::MyIp),
            "ipapicom" => Ok(LookupProvider::IpApiCom),
            "ipwhois" => Ok(LookupProvider::IpWhoIs),
            "ipapico" => Ok(LookupProvider::IpApiCo),
            "ipapiio" => Ok(LookupProvider::IpApiIo),
            "ipbase" => Ok(LookupProvider::IpBase),
            "iplocateio" => Ok(LookupProvider::IpLocateIo),
            "ipleak" => Ok(LookupProvider::IpLeak),
            "mullvad" => Ok(LookupProvider::Mullvad),
            "abstract" => Ok(LookupProvider::AbstractApi),
            "ipgeolocation" => Ok(LookupProvider::IpGeolocation),
            "ipdata" => Ok(LookupProvider::IpData),
            "ip2location" => Ok(LookupProvider::Ip2Location),
            "myipcom" => Ok(LookupProvider::MyIpCom),
            "ipify" => Ok(LookupProvider::Ipify),
            "getjsonip" => Ok(LookupProvider::GetJsonIp),
            "ipquery" => Ok(LookupProvider::IpQuery),
            _ => Err(LookupError::GenericError(format!(
                "Provider not found: {}",
                p
            ))),
        }
    }
}

impl LookupProvider {
    /// Builds the concrete lookup service out of a LookupProvider enum
    pub fn build(self) -> Box<dyn Provider + Send + Sync> {
        match self {
            LookupProvider::FreeIpApi => Box::new(freeipapi::FreeIpApi),
            LookupProvider::IfConfig => Box::new(ifconfig::IfConfig),
            LookupProvider::IpInfo => Box::new(ipinfo::IpInfo),
            LookupProvider::MyIp => Box::new(myip::MyIp),
            LookupProvider::IpApiCom => Box::new(ipapicom::IpApiCom),
            LookupProvider::IpApiCo => Box::new(ipapico::IpApiCo),
            LookupProvider::IpApiIo => Box::new(ipapiio::IpApiIo),
            LookupProvider::IpWhoIs => Box::new(ipwhois::IpWhoIs),
            LookupProvider::IpBase => Box::new(ipbase::IpBase),
            LookupProvider::IpLocateIo => Box::new(iplocateio::IpLocateIo),
            LookupProvider::IpLeak => Box::new(ipleak::IpLeak),
            LookupProvider::Mullvad => Box::new(mullvad::Mullvad),
            LookupProvider::AbstractApi => Box::new(abstractapi::AbstractApi),
            LookupProvider::IpGeolocation => Box::new(ipgeolocation::IpGeolocation),
            LookupProvider::IpData => Box::new(ipdata::IpData),
            LookupProvider::Ip2Location => Box::new(ip2location::Ip2Location),
            LookupProvider::MyIpCom => Box::new(myipcom::MyIpCom),
            LookupProvider::Ipify => Box::new(ipify::Ipify),
            LookupProvider::GetJsonIp => Box::new(getjsonip::GetJsonIp),
            LookupProvider::IpQuery => Box::new(ipquery::IpQuery),
            LookupProvider::Mock(ip) => Box::new(mock::Mock { ip }),
        }
    }

    /// Parse a `&str` into a LookupProvider with Parameters
    ///
    /// This function parses a `&str` into a LookupProvider enum variant and extracts the API key as parameter if it exists.
    /// The `&str` should be formatted as `<provider> <api_key>` or `<provider>`.
    pub fn from_str_with_params(s: &str) -> Result<(LookupProvider, Option<Parameters>)> {
        let s = s.trim().to_lowercase();
        // split the string into parts
        let s = s
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<String>>();
        // get the provider
        let p = s
            .first()
            .ok_or(LookupError::GenericError("No provider given".to_string()))?;
        let provider = p.parse::<LookupProvider>()?;
        // get the key if it exists
        let key = s.get(1).map(|key| Parameters::new(key.to_owned()));
        Ok((provider, key))
    }
}

/// Parameters hold the API key for lookup providers
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Parameters {
    /// API key for the lookup provider
    pub api_key: String,
}

impl Parameters {
    /// Creates new Parameters with an API key
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

/// LookupService instance to handle the lookup process
///
/// # Example
/// ```
/// use public_ip_address::lookup::{LookupProvider, LookupService};
///
/// let service = LookupService::new(LookupProvider::IpApiCom, None);
/// ```
#[non_exhaustive]
pub struct LookupService {
    provider: Box<dyn Provider + Send + Sync>,
    parameters: Option<Parameters>,
}

impl LookupService {
    /// Creates a new `LookupService` instance with parameters.
    pub fn new(provider: LookupProvider, parameters: Option<Parameters>) -> Self {
        LookupService {
            provider: provider.build(),
            parameters,
        }
    }

    /// Changes the provider for the LookupService
    pub fn set_provider(&mut self, provider: LookupProvider) -> &Self {
        self.provider = provider.build();
        self
    }

    /// Sets the parameters for the LookupService
    pub fn set_parameters(&mut self, parameters: Parameters) -> &Self {
        self.parameters = Some(parameters);
        self
    }

    /// Returns the type of the current lookup provider.
    ///
    /// This function returns the `LookupProvider` enum variant that represents the type of the current lookup provider.
    pub fn get_provider_type(&self) -> LookupProvider {
        self.provider.get_type()
    }

    /// Makes a request to the lookup provider
    ///
    /// This function makes an API request to the current lookup provider and parses the response into a `LookupResponse` instance.
    #[maybe_async::maybe_async]
    pub async fn lookup(&self, target: Option<IpAddr>) -> Result<LookupResponse> {
        if target.is_some() && !self.provider.supports_target_lookup() {
            return Err(LookupError::TargetNotSupported);
        }
        let response = self.make_api_request(target).await?;
        self.provider.parse_reply(response)
    }

    /// Internal function to make the API request
    #[maybe_async::maybe_async]
    async fn make_api_request(&self, target: Option<IpAddr>) -> Result<String> {
        let key = self.parameters.as_ref().map(|p| p.api_key.clone());
        let response = self.provider.get_client(key, target).send().await;
        handle_response(response).await
    }
}

/// Handles the response from reqwest
#[maybe_async::maybe_async]
pub async fn handle_response(response: reqwest::Result<Response>) -> Result<String> {
    match response {
        Ok(response) => match response.status() {
            StatusCode::OK => Ok(response.text().await?),
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
    use serial_test::serial;

    #[test]
    fn test_set_provider() {
        let mut provider = LookupService::new(LookupProvider::IpApiCom, None);
        assert_eq!(provider.get_provider_type(), LookupProvider::IpApiCom);
        provider.set_provider(LookupProvider::IpInfo);
        assert_eq!(provider.get_provider_type(), LookupProvider::IpInfo);
    }

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_make_request() {
        let address = "1.1.1.1".parse::<std::net::IpAddr>().unwrap();
        let provider = LookupService::new(LookupProvider::Mock(address.to_string()), None);
        let response = provider.lookup(None).await.unwrap();
        assert_eq!(response.ip, address);
    }

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    #[serial]
    async fn test_handle_response() {
        let response = client::get("https://httpbin.org/status/200").await;
        let body = handle_response(response).await;
        assert!(body.is_ok(), "Response is an error {:#?}", body);
    }

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    #[serial]
    async fn test_handle_response_error() {
        let response = client::get("https://httpbin.org/status/500").await;
        let body = handle_response(response).await;
        assert!(body.is_err(), "Response should be an error {:#?}", body);
        let body = body.unwrap_err();
        assert_eq!(
            body.to_string(),
            "Request status",
            "Wrong error {:#?}",
            body
        );
    }

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    #[serial]
    async fn test_handle_response_too_many() {
        let response = client::get("https://httpbin.org/status/429").await;
        let body = handle_response(response).await;
        assert!(body.is_err(), "Response should be an error {:#?}", body);
        let body = body.unwrap_err();
        assert_eq!(
            body.to_string(),
            "Too many API requests",
            "Wrong error {:#?}",
            body
        );
    }

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    #[serial]
    async fn test_target_supported() {
        let address = "8.8.8.8".parse::<std::net::IpAddr>().unwrap();
        let provider = LookupService::new(LookupProvider::Mock(address.to_string()), None);
        let response = provider.lookup(Some(address)).await;
        assert!(response.is_ok());
    }

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_target_not_supported() {
        let address = "1.1.1.1".parse::<std::net::IpAddr>().unwrap();
        let provider = LookupService::new(LookupProvider::MyIp, None);
        let response = provider.lookup(Some(address)).await.unwrap_err();
        assert_eq!(
            response.to_string(),
            "Target lookup not supported",
            "Target lookup should fail"
        );
    }

    #[test]
    fn test_conversions() {
        let provider = LookupProvider::from_str("freeipapi").unwrap();
        assert_eq!(provider, LookupProvider::FreeIpApi, "Conversion failed");

        let provider = LookupProvider::from_str("unknown");
        assert!(provider.is_err(), "Conversion should fail");
    }

    #[test]
    fn test_conversions_with_key() {
        let (provider, parameters) = LookupProvider::from_str_with_params("ipdata abc").unwrap();
        assert_eq!(provider, LookupProvider::IpData, "Conversion failed");
        assert_eq!(
            parameters,
            Some(Parameters {
                api_key: "abc".to_string()
            }),
            "Parameter conversion failed"
        );

        let (provider, parameters) = LookupProvider::from_str_with_params("ipdata").unwrap();
        assert_eq!(provider, LookupProvider::IpData, "Conversion failed");
        assert_eq!(parameters, None, "Parameter conversion failed");
    }
}
