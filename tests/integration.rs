use public_ip_address::*;
use public_ip_address::{cache::ResponseCache, lookup::LookupProvider};
use serial_test::serial;
use std::net::IpAddr;
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

fn clear_cache() {
    _ = ResponseCache::default().delete();
}

fn ip(ip: &str) -> IpAddr {
    ip.parse().unwrap()
}

#[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
#[serial]
async fn test_perform_lookup() {
    // Start a local mock server
    #[cfg(not(feature = "blocking"))]
    let server = setup_mock_server(200).await;
    #[cfg(feature = "blocking")]
    let (_rt, server) = setup_mock_server(200);

    let response = perform_lookup_with(
        vec![(
            LookupProvider::Mock("1.1.1.1".to_string(), server.uri()),
            None,
        )],
        None,
    )
    .await;
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().ip,
        ip("1.1.1.1"),
        "IP address not matching"
    );
}

#[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
#[serial]
async fn test_perform_lookup_target() {
    // Start a local mock server
    #[cfg(not(feature = "blocking"))]
    let server = setup_mock_server(200).await;
    #[cfg(feature = "blocking")]
    let (_rt, server) = setup_mock_server(200);

    let response = perform_lookup_with(
        vec![(
            LookupProvider::Mock("8.8.8.8".to_string(), server.uri()),
            None,
        )],
        Some(ip("8.8.8.8")),
    )
    .await;
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().ip,
        ip("8.8.8.8"),
        "IP address not matching"
    );
}

#[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
#[serial]
async fn test_perform_lookup_cached() {
    // Start a local mock server
    #[cfg(not(feature = "blocking"))]
    let server = setup_mock_server(200).await;
    #[cfg(feature = "blocking")]
    let (_rt, server) = setup_mock_server(200);

    clear_cache();
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("11.1.1.1".to_string(), server.uri()),
            None,
        )],
        None,
        Some(1),
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("11.1.1.1"),
        "IP address not matching"
    );
    clear_cache();
}

#[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
#[serial]
async fn test_perform_lookup_cached_force_expire() {
    // Start a local mock server
    #[cfg(not(feature = "blocking"))]
    let server = setup_mock_server(200).await;
    #[cfg(feature = "blocking")]
    let (_rt, server) = setup_mock_server(200);

    clear_cache();
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("21.1.1.1".to_string(), server.uri()),
            None,
        )],
        None,
        None,
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("21.1.1.1"),
        "IP address not matching"
    );
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("22.2.2.2".to_string(), server.uri()),
            None,
        )],
        None,
        Some(1),
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("21.1.1.1"),
        "Non expiring cache should be used"
    );
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("23.3.3.3".to_string(), server.uri()),
            None,
        )],
        None,
        Some(1),
        true,
    )
    .await;
    // the old cache should be flushed
    assert_eq!(
        response.unwrap().ip,
        ip("23.3.3.3"),
        "The old cache should be flushed"
    );
    clear_cache();
}

#[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
#[serial]
async fn test_perform_lookup_cached_expired() {
    // Start a local mock server
    #[cfg(not(feature = "blocking"))]
    let server = setup_mock_server(200).await;
    #[cfg(feature = "blocking")]
    let (_rt, server) = setup_mock_server(200);

    clear_cache();
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("1.1.1.1".to_string(), server.uri()),
            None,
        )],
        None,
        Some(1),
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("1.1.1.1"),
        "IP address not matching"
    );
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("2.2.2.2".to_string(), server.uri()),
            None,
        )],
        None,
        Some(2),
        false,
    )
    .await;
    // the old cache should be returned
    assert_eq!(
        response.unwrap().ip,
        ip("1.1.1.1"),
        "The old cache should be returned"
    );
    std::thread::sleep(std::time::Duration::from_secs(1));
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("3.3.3.3".to_string(), server.uri()),
            None,
        )],
        None,
        Some(0),
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("3.3.3.3"),
        "Cached value should expire"
    );

    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("4.4.4.4".to_string(), server.uri()),
            None,
        )],
        None,
        Some(1),
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("4.4.4.4"),
        "Cached value should expire"
    );
    let response = perform_cached_lookup_with(
        vec![(
            LookupProvider::Mock("5.5.5.5".to_string(), server.uri()),
            None,
        )],
        None,
        Some(1),
        false,
    )
    .await;
    assert_eq!(
        response.unwrap().ip,
        ip("4.4.4.4"),
        "Cached value should be used"
    );
    clear_cache();
}
