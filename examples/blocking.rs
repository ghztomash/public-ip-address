use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Perform my public IP address lookup
    let result = public_ip_address::perform_lookup(None)?;
    println!("{}", result);
    Ok(())
}
