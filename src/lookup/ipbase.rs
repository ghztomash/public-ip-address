//! <https://ipbase.com> lookup provider

use super::{client::RequestBuilder, ProviderResponse, Result};
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// <https://ipbase.com/docs/info>
#[derive(Serialize, Deserialize, Debug)]
pub struct IpBaseResponse {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    ip: String,
    hostname: Option<String>,
    #[serde(rename = "type")]
    ip_type: Option<String>,
    connection: Option<Connection>,
    location: Option<Location>,
    timezone: Option<Timezone>,
    security: Option<Security>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Connection {
    asn: Option<i64>,
    organization: Option<String>,
    isp: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    latitude: Option<f64>,
    longitude: Option<f64>,
    zip: Option<String>,
    continent: Option<Continent>,
    country: Option<Country>,
    city: Option<City>,
    region: Option<Region>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Continent {
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Country {
    #[serde(rename = "alpha2")]
    code: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct City {
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Region {
    #[serde(rename = "alpha2")]
    code: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timezone {
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Security {
    is_proxy: Option<bool>,
    is_vpn: Option<bool>,
    is_tor: Option<bool>,
}

impl ProviderResponse<IpBaseResponse> for IpBaseResponse {
    fn into_response(self) -> LookupResponse {
        let data = self.data;
        let mut response = LookupResponse::new(
            data.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpBase,
        );
        response.hostname = data.hostname;
        if let Some(connection) = data.connection {
            response.asn_org = connection.organization;
            if let Some(number) = connection.asn {
                response.asn = Some(format!("{number}"));
            }
        }

        if let Some(location) = data.location {
            response.latitude = location.latitude;
            response.longitude = location.longitude;
            if let Some(country) = location.country {
                response.country = country.name;
                response.country_code = country.code;
            }
            if let Some(city) = location.city {
                response.city = city.name;
            }
            if let Some(region) = location.region {
                response.region = region.name;
            }
        }

        if let Some(timezone) = data.timezone {
            response.time_zone = timezone.id;
        }

        if let Some(risk) = data.security {
            let is_proxy = risk.is_proxy.unwrap_or(false)
                || risk.is_vpn.unwrap_or(false)
                || risk.is_tor.unwrap_or(false);
            response.is_proxy = Some(is_proxy);
        }

        response
    }
}

/// IpBase lookup provider
pub struct IpBase;

impl Provider for IpBase {
    fn get_endpoint(&self, _key: &Option<String>, target: &Option<IpAddr>) -> String {
        let target = match target.map(|t| t.to_string()) {
            Some(t) => format!("?ip={t}"),
            None => "".to_string(),
        };
        format!("https://api.ipbase.com/v2/info{target}")
    }

    fn add_auth(&self, request: RequestBuilder, key: &Option<String>) -> RequestBuilder {
        if let Some(key) = key {
            return request.header("apikey", key);
        }
        request
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpBaseResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpBase
    }

    fn supports_target_lookup(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
    "data": {
        "ip": "1.1.1.1",
        "hostname": "one.one.one.one",
        "type": "v4",
        "range_type": {
            "type": "PUBLIC",
            "description": "Public address"
        },
        "connection": {
            "asn": 13335,
            "organization": "Cloudflare, Inc.",
            "isp": "APNIC Research and Development",
            "range": "1.1.1.1/32"
        },
        "location": {
            "geonames_id": 5368753,
            "latitude": 34.053611755371094,
            "longitude": -118.24549865722656,
            "zip": "90012",
            "continent": {
                "code": "NA",
                "name": "North America",
                "name_translated": "North America",
                "geonames_id": 6255149,
                "wikidata_id": "Q49"
            },
            "country": {
                "alpha2": "US",
                "alpha3": "USA",
                "calling_codes": [
                    "+1"
                ],
                "currencies": [
                    {
                        "symbol": "$",
                        "name": "US Dollar",
                        "symbol_native": "$",
                        "decimal_digits": 2,
                        "rounding": 0,
                        "code": "USD",
                        "name_plural": "US dollars"
                    }
                ],
                "emoji": "🇺🇸",
                "ioc": "USA",
                "languages": [
                    {
                        "name": "English",
                        "name_native": "English"
                    }
                ],
                "name": "United States",
                "name_translated": "United States",
                "timezones": [
                    "America/New_York",
                    "America/Detroit",
                    "America/Kentucky/Louisville",
                    "America/Kentucky/Monticello",
                    "America/Indiana/Indianapolis",
                    "America/Indiana/Vincennes",
                    "America/Indiana/Winamac",
                    "America/Indiana/Marengo",
                    "America/Indiana/Petersburg",
                    "America/Indiana/Vevay",
                    "America/Chicago",
                    "America/Indiana/Tell_City",
                    "America/Indiana/Knox",
                    "America/Menominee",
                    "America/North_Dakota/Center",
                    "America/North_Dakota/New_Salem",
                    "America/North_Dakota/Beulah",
                    "America/Denver",
                    "America/Boise",
                    "America/Phoenix",
                    "America/Los_Angeles",
                    "America/Anchorage",
                    "America/Juneau",
                    "America/Sitka",
                    "America/Metlakatla",
                    "America/Yakutat",
                    "America/Nome",
                    "America/Adak",
                    "Pacific/Honolulu"
                ],
                "is_in_european_union": false,
                "fips": "US",
                "geonames_id": 6252001,
                "hasc_id": "US",
                "wikidata_id": "Q30"
            },
            "city": {
                "fips": "0644000",
                "alpha2": null,
                "geonames_id": 5368753,
                "hasc_id": null,
                "wikidata_id": "Q65",
                "name": "Los Angeles",
                "name_translated": "Los Angeles"
            },
            "region": {
                "fips": "US06",
                "alpha2": "US-CA",
                "geonames_id": 5332921,
                "hasc_id": "US.CA",
                "wikidata_id": "Q99",
                "name": "California",
                "name_translated": "California"
            }
        },
        "tlds": [
            ".us"
        ],
        "timezone": {
            "id": "America/Los_Angeles",
            "current_time": "2023-06-28T07:46:37-07:00",
            "code": "PDT",
            "is_daylight_saving": true,
            "gmt_offset": -25200
        },
        "security": {
            "is_anonymous": false,
            "is_datacenter": false,
            "is_vpn": false,
            "is_bot": false,
            "is_abuser": false,
            "is_known_attacker": false,
            "is_proxy": false,
            "is_spam": false,
            "is_tor": false,
            "is_icloud_relay": false,
            "threat_score": 100
        },
        "domains": {
            "count": 12337,
            "domains": [
                "eliwise.academy",
                "accountingprose.academy",
                "1and1-test-ntlds-fr.accountant",
                "sulphur.africa",
                "saadeh.agency"
            ]
        }
    }
}
"#;

    #[ignore]
    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn test_request() {
        let service = Box::new(IpBase);
        let result = service.get_client(None, None).send().await;
        let result = super::super::handle_response(result).await.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpBase: {result:#?}");
        let response = IpBaseResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {response:#?}");
    }

    #[test]
    fn test_parse() {
        let response = IpBaseResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.data.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
