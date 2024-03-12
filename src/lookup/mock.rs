use crate::lookup::LookupService;
use crate::LookupResponse;
use crate::Result;

pub struct Mock<'a> {
    pub ip: &'a str,
}

impl LookupService for Mock<'_> {
    fn make_api_request(&self) -> Result<String> {
        Ok(self.ip.to_string())
    }
    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        Ok(LookupResponse::new(json))
    }
}
