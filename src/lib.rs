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
    longitude: f32,
    time_zone: String,
    asn: String,
    asn_org: String,
}

fn parse_response(input: String) -> Result<Response, Error> {
    let deserialized: Response = serde_json::from_str(&input).unwrap();
    Ok(deserialized)
}

fn make_api_request() -> Result<String, Error> {
    let response = reqwest::blocking::get("http://ifconfig.co/json")?.text()?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request() {
        let result = make_api_request();
        assert!(result.is_ok(), "Failed getting result");
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("{:#?}", result);
    }

    #[test]
    fn test_parse() {
        let input = "{\n  \"ip\": \"1.1.1.1\",\n  \"ip_decimal\": 16843009,\n  \"country\": \"Germany\",\n  \"country_iso\": \"DE\",\n  \"country_eu\": true,\n  \"region_name\": \"Hesse\",\n  \"region_code\": \"HE\",\n  \"zip_code\": \"60326\",\n  \"city\": \"Frankfurt am Main\",\n  \"latitude\": 50.1049,\n  \"longitude\": 8.6295,\n  \"time_zone\": \"Europe/Berlin\",\n  \"asn\": \"AS9009\",\n  \"asn_org\": \"M247 Europe SRL\"\n}".to_string();
        let response = parse_response(input).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
    }
}
