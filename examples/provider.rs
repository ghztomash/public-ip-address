use public_ip_address::lookup::{
    ipwhois::{IpWhoIs, IpWhoIsResponse},
    Provider,
};
use std::error::Error;

/// This example demonstrates how to use the IpWhoIs provider directly
/// and get access to the provider specific response.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let provider = IpWhoIs;
    let response = provider.make_api_request(None, None).await?;
    let result = IpWhoIsResponse::parse(response)?;
    println!("{:#?}", result);
    Ok(())
}
