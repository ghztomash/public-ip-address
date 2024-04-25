//! Mock lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use std::net::IpAddr;

/// Mock lookup provider
pub struct Mock {
    /// IP address to return
    pub ip: String,
}

impl Provider for Mock {
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        "https://httpbin.org/status/200".to_string()
    }

    fn parse_reply(&self, _json: String) -> Result<LookupResponse> {
        Ok(LookupResponse::new(
            self.ip.parse::<std::net::IpAddr>().unwrap(),
            LookupProvider::Mock(self.ip.to_string()),
        ))
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Mock(self.ip.to_string())
    }

    fn supports_target_lookup(&self) -> bool {
        true
    }
}
