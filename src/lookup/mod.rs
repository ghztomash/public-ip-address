use crate::LookupResponse;
use crate::Result;
use reqwest::blocking::Response;
use reqwest::StatusCode;

pub mod freeipapi;
pub mod ifconfig;
pub mod ipapi;
pub mod ipinfo;
pub mod mock;
pub mod myip;

pub trait LookupService {
    fn make_api_request(&self) -> Result<String>;
    fn parse_reply(&self, json: String) -> Result<LookupResponse>;
}

pub struct Service {
    provider: Box<dyn LookupService>,
}

pub enum LookupProvider {
    FreeIpApi,
    IfConfig,
    IpInfo,
    MyIp,
    IpApi,
    Mock(String),
}

impl Service {
    pub fn new(provider: LookupProvider) -> Self {
        let provider: Box<dyn LookupService> = match provider {
            LookupProvider::FreeIpApi => Box::new(freeipapi::FreeIpApi),
            LookupProvider::IfConfig => Box::new(ifconfig::IfConfig),
            LookupProvider::IpInfo => Box::new(ipinfo::IpInfo),
            LookupProvider::MyIp => Box::new(myip::MyIp),
            LookupProvider::IpApi => Box::new(ipapi::IpApi),
            LookupProvider::Mock(ip) => Box::new(mock::Mock { ip }),
        };
        Service { provider }
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
            StatusCode::TOO_MANY_REQUESTS => {
                Err(format!("Too many requests: {}", response.status()).into())
            }
            s => Err(format!("Status: {}", s).into()),
        },
        Err(e) => Err(format!("Error GET request: {}", e).into()),
    }
}
