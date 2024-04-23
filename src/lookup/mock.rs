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

#[async_trait::async_trait]
impl Provider for Mock {
    #[inline]
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        "https://httpbin.org/status/200".to_string()
    }

    async fn make_api_request(
        &self,
        _key: Option<String>,
        target: Option<IpAddr>,
    ) -> Result<String> {
        let target = target.map(|t| t.to_string());
        // simulate blocking api call
        thread::sleep(time::Duration::from_millis(100));
        Ok(target.unwrap_or(self.ip.to_owned()))
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
}
