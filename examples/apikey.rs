use public_ip_address::lookup::{LookupProvider, LookupService, Parameters};
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    // read the API key from the environment variables
    let key = env::var("ABSTRACT_APIKEY").ok();
    let provider = LookupProvider::AbstractApi;
    let parameters = Parameters::new(key);
    let service = LookupService::new(provider, parameters);
    let result = service.lookup(None)?;
    println!("{}", result);
    Ok(())
}
