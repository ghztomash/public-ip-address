use crate::LookupResponse;
use crate::Result;
use reqwest::StatusCode;

pub mod freeipapi;
pub mod ifconfig;
pub mod mock;

pub trait LookupService {
    fn make_api_request(&self) -> Result<String>;
    fn parse_reply(&self, json: String) -> Result<LookupResponse>;
}

pub struct Service {
    provider: Box<dyn LookupService>,
}

impl Service {
    pub fn new(provider: Box<dyn LookupService>) -> Self {
        Service { provider }
    }

    pub fn make_request(&self) -> Result<LookupResponse> {
        let response = self.provider.make_api_request()?;
        self.provider.parse_reply(response)
    }
}

fn handle_response(response: reqwest::Result<reqwest::blocking::Response>) -> Result<String> {
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
