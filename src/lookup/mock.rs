use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use std::{thread, time};

pub struct Mock {
    pub ip: String,
}

impl Provider for Mock {
    fn make_api_request(&self) -> Result<String> {
        // simulate blocking api call
        thread::sleep(time::Duration::from_millis(100));
        Ok(self.ip.to_string())
    }
    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        Ok(LookupResponse::new(
            json.to_string(),
            LookupProvider::Mock(json),
        ))
    }
    fn get_type(&self) -> LookupProvider {
        LookupProvider::Mock(self.ip.to_string())
    }
}
