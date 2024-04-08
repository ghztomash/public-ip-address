use public_ip_address::lookup::{LookupProvider, LookupService};
use std::{error::Error, str::FromStr};

fn main() -> Result<(), Box<dyn Error>> {
    let provider = LookupProvider::from_str("ipinfo")?;
    let parameters = None;
    let service = LookupService::new(provider, parameters);
    let result = service.lookup(None)?;
    println!("{}", result);
    Ok(())
}
