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
    /// Endpoint to mock
    pub endpoint: String,
}

impl Provider for Mock {
    fn get_endpoint(&self, _key: &Option<String>, _target: &Option<IpAddr>) -> String {
        self.endpoint.clone()
    }

    fn parse_reply(&self, _json: String) -> Result<LookupResponse> {
        Ok(LookupResponse::new(
            self.ip.parse::<std::net::IpAddr>().unwrap(),
            LookupProvider::Mock(self.ip.clone(), self.endpoint.clone()),
        ))
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::Mock(self.ip.clone(), self.endpoint.clone())
    }

    fn supports_target_lookup(&self) -> bool {
        true
    }
}

/// Helper module for mock API
#[cfg(test)]
pub mod helper {
    use wiremock::{matchers::method, Mock as WireMock, MockServer, ResponseTemplate};

    /// Setup mock API endpoint
    #[cfg(not(feature = "blocking"))]
    pub async fn setup_mock_server(status_code: u16) -> MockServer {
        let server = MockServer::start().await;

        let resp = ResponseTemplate::new(status_code);

        WireMock::given(method("GET"))
            .respond_with(resp)
            .mount(&server)
            .await;

        server
    }

    /// Setup mock API endpoint
    /// In blocking builds, provide a sync API that internally spins a Tokio runtime.
    #[cfg(feature = "blocking")]
    pub fn setup_mock_server(code: u16) -> (tokio::runtime::Runtime, MockServer) {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let server = rt.block_on(async {
            let s = MockServer::start().await;

            let resp = ResponseTemplate::new(code);

            WireMock::given(method("GET"))
                .respond_with(resp)
                .mount(&s)
                .await;

            s
        });

        (rt, server)
    }
}
