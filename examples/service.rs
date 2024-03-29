use public_ip_address::lookup::{LookupProvider, LookupService};
use std::{error::Error, str::FromStr};

fn main() -> Result<(), Box<dyn Error>> {
    let provider = LookupProvider::from_str("ipinfo")?;
    let service = LookupService::new(provider);
    let result = service.make_request()?;
    println!("{}", result);
    Ok(())
}
