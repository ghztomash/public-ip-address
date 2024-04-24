use public_ip_address::lookup::{LookupProvider, LookupService};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let (provider, parameters) = LookupProvider::from_str_with_params("ipinfo")?;
    let service = LookupService::new(provider, parameters);
    let result = service.lookup(None).await?;
    println!("{}", result);
    Ok(())
}
