use public_ip_address::lookup::LookupProvider;
use std::error::Error;

#[cfg_attr(not(feature = "blocking"), tokio::main)]
#[maybe_async::maybe_async]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let result = public_ip_address::perform_cached_lookup_with(
        vec![(LookupProvider::MyIp, None)],
        None,
        Some(2),
        false,
    )
    .await?;
    println!(
        "Hello {} from {}, {}.",
        result.ip,
        result.city.unwrap_or("unknown".to_string()),
        result.country.unwrap_or("unknown".to_string())
    );
    Ok(())
}
