use public_ip_address::lookup::{
    handle_response,
    ipwhois::{IpWhoIs, IpWhoIsResponse},
    Provider, ProviderResponse,
};
use std::error::Error;

/// This example demonstrates how to use the IpWhoIs provider directly
/// and get access to the provider specific response.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let provider = IpWhoIs;
    let response = provider.get_client(None, None).send().await;
    let response = handle_response(response).await?;
    let result = IpWhoIsResponse::parse(response)?;
    println!("{:#?}", result);
    Ok(())
}
