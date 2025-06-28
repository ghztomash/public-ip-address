use public_ip_address::lookup::{LookupProvider, LookupService};
use std::error::Error;

#[cfg_attr(not(feature = "blocking"), tokio::main)]
#[maybe_async::maybe_async]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let (provider, parameters) = LookupProvider::from_str_with_params("ipinfo")?;
    let service = LookupService::new(provider, parameters);
    let result = service.lookup(None).await?;
    println!("{result}");
    Ok(())
}
