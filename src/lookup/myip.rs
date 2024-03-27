use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};

// https://www.my-ip.io/api-usage
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MyIpResponse {
    success: bool,
    ip: String,
    #[serde(rename = "type")]
    ip_type: Option<String>,
    country: Option<Country>,
    region: Option<String>,
    city: Option<String>,
    location: Option<Location>,
    time_zone: Option<String>,
    asn: Option<Asn>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Country {
    code: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    lat: Option<f64>,
    lon: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Asn {
    number: Option<i64>,
    name: Option<String>,
    network: Option<String>,
}

impl MyIpResponse {
    pub fn parse(input: String) -> Result<MyIpResponse> {
        let deserialized: MyIpResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn convert(&self) -> LookupResponse {
        let mut response = LookupResponse::new(self.ip.clone(), LookupProvider::MyIp);
        if let Some(country) = &self.country {
            response.country = country.name.clone();
            response.country_code = country.code.clone();
        }
        response.region = self.region.clone();
        response.city = self.city.clone();
        if let Some(location) = &self.location {
            response.latitude = location.lat;
            response.longitude = location.lon;
        }
        response.time_zone = self.time_zone.clone();
        if let Some(asn) = &self.asn {
            response.asn_org = asn.name.clone();
            if let Some(number) = asn.number {
                response.asn = Some(format!("{number}"));
            }
        }
        response
    }
}

pub struct MyIp;
impl Provider for MyIp {
    fn make_api_request(&self) -> Result<String> {
        let response = reqwest::blocking::get("https://api.my-ip.io/v2/ip.json");
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = MyIpResponse::parse(json)?;
        Ok(response.convert())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::MyIp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "success": true,
  "ip": "1.1.1.1",
  "type": "IPv4",
  "country": {
    "code": "DE",
    "name": "Germany"
  },
  "region": "Bavaria",
  "city": "Gunzenhausen",
  "location": {
    "lat": 49.1156,
    "lon": 10.7511
  },
  "timeZone": "Europe/Berlin",
  "asn": {
    "number": 24940,
    "name": "Hetzner Online GmbH",
    "network": "23.88.0.0/17"
  }
}
"#;

    #[test]
    fn test_request() {
        let service = Box::new(MyIp);
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result");
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("MyIp: {:#?}", result);
        let response = MyIpResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response");
    }

    #[test]
    fn test_parse() {
        let response = MyIpResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.convert();
        assert_eq!(lookup.ip, "1.1.1.1", "IP address not matching");
    }
}
