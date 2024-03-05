use serde::{Deserialize, Serialize};
use reqwest::Error;

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    ip: String,
    ip_decimal: u32,
    country: String,
    country_iso: String,
    country_eu: bool,
    region_name: String,
    region_code: String,
    zip_code: String,
    city: String,
    latitude: f32,
    longtitude: f32,
    time_zone: String,
    asn: String,
    asn_org: String,
}

fn make_api_request() -> Result<String, Error> {
    let resp = reqwest::blocking::get("http://ifconfig.co/json")?.text()?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request() {
        let result = make_api_request();
        println!("{:#?}", result);
        assert!(result.is_ok());
    }
}
