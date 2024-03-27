use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};

// https://ip-api.com/docs/api:json
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IpApiResponse {
    query: String,
    status: Option<String>,
    continent: Option<String>,
    continent_code: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    region: Option<String>,
    region_name: Option<String>,
    city: Option<String>,
    district: Option<String>,
    zip: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<String>,
    offset: Option<i64>,
    currency: Option<String>,
    isp: Option<String>,
    org: Option<String>,
    #[serde(rename = "as")]
    asn: Option<String>,
    as_name: Option<String>,
    reverse: Option<String>,
    mobile: Option<bool>,
    proxy: Option<bool>,
    hosting: Option<bool>,
}

impl IpApiResponse {
    pub fn parse(input: String) -> Result<IpApiResponse> {
        let deserialized: IpApiResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(self.query, LookupProvider::IpApi);
        response.country = self.country;
        response.country_code = self.country_code;
        response.region = self.region_name;
        response.region_code = self.region;
        response.postal_code = self.zip;
        response.city = self.city;
        response.latitude = self.lat;
        response.longitude = self.lon;
        response.time_zone = self.timezone;
        response.asn_org = self.org;
        response.asn = self.asn;
        response.hostname = self.reverse;
        response.proxy = self.proxy;
        response
    }
}

pub struct IpApi;
impl Provider for IpApi {
    fn make_api_request(&self) -> Result<String> {
        let response = reqwest::blocking::get("http://ip-api.com/json?fields=66846719");
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpApiResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpApi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "query": "1.1.1.1",
  "status": "success",
  "continent": "North America",
  "continentCode": "NA",
  "country": "Canada",
  "countryCode": "CA",
  "region": "QC",
  "regionName": "Quebec",
  "city": "Montreal",
  "district": "",
  "zip": "H1K",
  "lat": 45.6085,
  "lon": -73.5493,
  "timezone": "America/Toronto",
  "offset": -14400,
  "currency": "CAD",
  "isp": "Le Groupe Videotron Ltee",
  "org": "Videotron Ltee",
  "as": "AS5769 Videotron Ltee",
  "asname": "VIDEOTRON",
  "reverse": "modemcable001.0-48-24.mc.videotron.ca",
  "mobile": false,
  "proxy": false,
  "hosting": false
}
"#;

    #[test]
    #[ignore]
    fn test_request() {
        let service = Box::new(IpApi);
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpApi: {:#?}", result);
        let response = IpApiResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpApiResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.query, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(lookup.ip, "1.1.1.1", "IP address not matching");
    }
}
