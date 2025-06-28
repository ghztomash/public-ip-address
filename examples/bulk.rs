use public_ip_address::lookup::{LookupProvider, LookupService};
use std::error::Error;

#[cfg_attr(not(feature = "blocking"), tokio::main)]
#[maybe_async::maybe_async]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let provider = LookupProvider::IpQuery;
    // create a lookup service
    let service = LookupService::new(provider, None);
    // targets for bulk lookup
    let targets = vec![
        "1.1.1.1".parse::<std::net::IpAddr>().unwrap(),
        "8.8.8.8".parse::<std::net::IpAddr>().unwrap(),
    ];
    // lookup target addresses
    let result = service.lookup_bulk(&targets).await?;
    println!("{:#?}", result);
    Ok(())
}
