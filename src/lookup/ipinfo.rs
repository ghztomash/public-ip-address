use crate::lookup::{handle_response, Provider};
use crate::LookupResponse;
use crate::Result;
use serde::{Deserialize, Serialize};

// https://ipinfo.io/json
#[derive(Serialize, Deserialize, Debug)]
pub struct IpInfoResponse {
    ip: String,
    hostname: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    loc: Option<String>,
    org: Option<String>,
    postal: Option<String>,
    timezone: Option<String>,
    readme: Option<String>,
}

impl IpInfoResponse {
    pub fn parse(input: String) -> Result<IpInfoResponse> {
        let deserialized: IpInfoResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn convert(&self) -> LookupResponse {
        let mut latitude = None;
        let mut longitude = None;

        // convert loc string to float
        if let Some(loc) = self.loc.clone() {
            let coords: Vec<&str> = loc.split(',').collect();
            if coords.len() == 2 {
                latitude = coords[0].parse().ok();
                longitude = coords[1].parse().ok();
            }
        }

        let mut response = LookupResponse::new(self.ip.clone());
        response.country = self.country.clone();
        response.country_iso = self.country.clone();
        response.region_name = self.region.clone();
        response.zip_code = self.postal.clone();
        response.city = self.city.clone();
        response.latitude = latitude;
        response.longitude = longitude;
        response.time_zone = self.timezone.clone();
        response.asn_org = self.org.clone();
        response.asn = self.org.clone();
        response
    }
}

pub struct IpInfo;
impl Provider for IpInfo {
    fn make_api_request(&self) -> Result<String> {
        let response = reqwest::blocking::get("https://ipinfo.io/json");
        handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpInfoResponse::parse(json)?;
        Ok(response.convert())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "ip": "1.1.1.1",
  "hostname": "ip-66-87-125-72.spfdma.spcsdns.net",
  "city": "Springfield",
  "region": "Massachusetts",
  "country": "US",
  "loc": "42.1015,-72.5898",
  "org": "AS10507 Sprint Personal Communications Systems",
  "postal": "01101",
  "timezone": "America/New_York"
}
"#;

    #[test]
    fn test_request() {
        let service = Box::new(IpInfo);
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result");
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpInfo: {:#?}", result);
    }

    #[test]
    fn test_parse() {
        let response = IpInfoResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
    }
}
