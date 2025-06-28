use public_ip_address::*;
use public_ip_address::{cache::ResponseCache, lookup::LookupProvider};
use serial_test::serial;
use std::net::IpAddr;

fn clear_cache() {
    _ = ResponseCache::default().delete();
}

fn ip(ip: &str) -> IpAddr {
    ip.parse().unwrap()
}

#[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
#[serial]
async fn test_perform_lookup() {
    let response = perform_lookup_with(
        vec![(LookupProvider::Mock("1.1.1.1".to_string()), None)],
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
    let response = perform_lookup_with(
        vec![(LookupProvider::Mock("8.8.8.8".to_string()), None)],
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
    clear_cache();
    let response = perform_cached_lookup_with(
        vec![(LookupProvider::Mock("11.1.1.1".to_string()), None)],
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
    clear_cache();
    let response = perform_cached_lookup_with(
        vec![(LookupProvider::Mock("21.1.1.1".to_string()), None)],
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
        vec![(LookupProvider::Mock("22.2.2.2".to_string()), None)],
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
        vec![(LookupProvider::Mock("23.3.3.3".to_string()), None)],
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
    clear_cache();
    let response = perform_cached_lookup_with(
        vec![(LookupProvider::Mock("1.1.1.1".to_string()), None)],
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
        vec![(LookupProvider::Mock("2.2.2.2".to_string()), None)],
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
        vec![(LookupProvider::Mock("3.3.3.3".to_string()), None)],
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
        vec![(LookupProvider::Mock("4.4.4.4".to_string()), None)],
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
        vec![(LookupProvider::Mock("5.5.5.5".to_string()), None)],
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
