//! Mock lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use std::net::IpAddr;
use std::{thread, time};

pub struct Mock {
    pub ip: String,
}

impl Provider for Mock {
    fn make_api_request(&self, _key: Option<String>, _target: Option<IpAddr>) -> Result<String> {
        // simulate blocking api call
        thread::sleep(time::Duration::from_millis(100));
        Ok(self.ip.to_owned())
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        Ok(LookupResponse::new(
            json.parse::<std::net::IpAddr>().unwrap(),
            LookupProvider::Mock(json),
        ))
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Mock(self.ip.to_string())
    }
}
