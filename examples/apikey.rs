use public_ip_address::lookup::{LookupProvider, LookupService};
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    // read the API key from the environment variables
    let key = env::var("ABSTRACT_APIKEY").ok();
    let provider = LookupProvider::AbstractApi(key);
    let service = LookupService::new(provider);
    let result = service.make_request()?;
    println!("{}", result);
    Ok(())
}
