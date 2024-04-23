use public_ip_address::lookup::{LookupProvider, LookupService, Parameters};
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // read the API key from the environment variables
    let key = env::var("ABSTRACT_APIKEY")?;
    let provider = LookupProvider::AbstractApi;
    // set the API key as a parameter
    let parameters = Some(Parameters::new(key));
    // create a lookup service
    let service = LookupService::new(provider, parameters);
    // lookup own public IP address
    let result = service.lookup(None).await?;
    println!("{}", result);
    Ok(())
}
