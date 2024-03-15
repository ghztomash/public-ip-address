use crate::lookup::Provider;
use crate::LookupResponse;
use crate::Result;
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
        Ok(LookupResponse::new(json))
    }
}
