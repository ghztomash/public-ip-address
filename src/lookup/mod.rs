use crate::LookupResponse;
use crate::Result;

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
