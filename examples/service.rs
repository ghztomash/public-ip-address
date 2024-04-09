use public_ip_address::lookup::{LookupProvider, LookupService};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let (provider, parameters) = LookupProvider::from_str_with_params("ipinfo")?;
    let service = LookupService::new(provider, parameters);
    let result = service.lookup(None)?;
    println!("{}", result);
    Ok(())
}
