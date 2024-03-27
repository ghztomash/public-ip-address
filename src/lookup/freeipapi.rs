use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};

// https://docs.freeipapi.com/response.html
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FreeIpApiResponse {
    ip_version: u8,
    ip_address: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    country_name: Option<String>,
    country_code: Option<String>,
    time_zone: Option<String>,
    zip_code: Option<String>,
    city_name: Option<String>,
    region_name: Option<String>,
    continent: Option<String>,
    continent_code: Option<String>,
    is_proxy: Option<bool>,
}

impl FreeIpApiResponse {
    pub fn parse(input: String) -> Result<FreeIpApiResponse> {
        let deserialized: FreeIpApiResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn convert(&self) -> LookupResponse {
        let mut response = LookupResponse::new(self.ip_address.clone(), LookupProvider::FreeIpApi);
        response.country = self.country_name.clone();
        response.country_code = self.country_code.clone();
        response.continent = self.continent.clone();
        response.region = self.region_name.clone();
        response.postal_code = self.zip_code.clone();
        response.city = self.city_name.clone();
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        response.time_zone = self.time_zone.clone();
        response.proxy = self.is_proxy;
        response
    }
}

pub struct FreeIpApi;
impl Provider for FreeIpApi {
    fn make_api_request(&self) -> Result<String> {
        let response = reqwest::blocking::get("https://freeipapi.com/api/json");
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = FreeIpApiResponse::parse(json)?;
        Ok(response.convert())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::FreeIpApi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
 "ipVersion": 4,
 "ipAddress": "1.1.1.1",
 "latitude": 58.416588,
 "longitude": 15.616713,
 "countryName": "Sweden",
 "countryCode": "SE",
 "timeZone": "+02:00",
 "zipCode": "58957",
 "cityName": "Linkoping",
 "regionName": "Ostergotlands lan",
 "continent": "Europe",
 "continentCode": "EU"
}
"#;

    #[test]
    fn test_request() {
        let service = Box::new(FreeIpApi);
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result");
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("FreeIpApi: {:#?}", result);
        let response = FreeIpApiResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response");
    }

    #[test]
    fn test_parse() {
        let response = FreeIpApiResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip_address, "1.1.1.1", "IP address not matching");
        let lookup = response.convert();
        assert_eq!(lookup.ip, "1.1.1.1", "IP address not matching");
    }
}
