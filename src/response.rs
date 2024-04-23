//! ✉️ Lookup response.

use crate::lookup::LookupProvider;
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr};

/// Lookup response containing information like IP, country, city, hostname etc.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct LookupResponse {
    /// Public IP address.
    pub ip: IpAddr,
    /// Continent name.
    pub continent: Option<String>,
    /// Country name.
    pub country: Option<String>,
    /// Country ISO code.
    pub country_code: Option<String>,
    /// Region name.
    pub region: Option<String>,
    /// Postal code.
    pub postal_code: Option<String>,
    /// City name.
    pub city: Option<String>,
    /// Latitude of the IP address.
    pub latitude: Option<f64>,
    /// Longitude of the IP address.
    pub longitude: Option<f64>,
    /// Time zone of the IP address.
    pub time_zone: Option<String>,
    /// Autonomous System Number.
    pub asn: Option<String>,
    /// Autonomous System Organization.
    pub asn_org: Option<String>,
    /// Hostname of the IP address.
    pub hostname: Option<String>,
    /// Is the IP a proxy or vpn?
    pub is_proxy: Option<bool>,
    /// Provider that was used for the lookup.
    pub provider: LookupProvider,
}

impl LookupResponse {
    /// Create a new lookup response.
    pub fn new(ip: IpAddr, provider: LookupProvider) -> Self {
        LookupResponse {
            ip,
            continent: None,
            country: None,
            country_code: None,
            region: None,
            postal_code: None,
            city: None,
            latitude: None,
            longitude: None,
            time_zone: None,
            asn: None,
            asn_org: None,
            hostname: None,
            is_proxy: None,
            provider,
        }
    }
}

impl fmt::Display for LookupResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "IP: {}", self.ip)?;
        if let Some(continent) = &self.continent {
            writeln!(f, "Continent: {}", continent)?;
        }
        if let Some(country) = &self.country {
            write!(f, "Country: {}", country)?;
        }
        if let Some(country_code) = &self.country_code {
            writeln!(f, " ({})", country_code)?;
        } else {
            writeln!(f)?;
        }
        if let Some(region) = &self.region {
            writeln!(f, "Region: {}", region)?;
        }
        if let Some(postal_code) = &self.postal_code {
            writeln!(f, "Postal code: {}", postal_code)?;
        }
        if let Some(city) = &self.city {
            writeln!(f, "City: {}", city)?;
        }
        if let Some(latitude) = &self.latitude {
            write!(f, "Coordinates: {}", latitude)?;
        }
        if let Some(longitude) = &self.longitude {
            writeln!(f, ", {}", longitude)?;
        } else {
            writeln!(f)?;
        }
        if let Some(time_zone) = &self.time_zone {
            writeln!(f, "Time zone: {}", time_zone)?;
        }
        if let Some(asn_org) = &self.asn_org {
            write!(f, "Organization: {}", asn_org)?;
        }
        if let Some(asn) = &self.asn {
            writeln!(f, " ({})", asn)?;
        } else {
            writeln!(f)?;
        }
        if let Some(hostname) = &self.hostname {
            writeln!(f, "Hostname: {}", hostname)?;
        }
        if let Some(proxy) = &self.is_proxy {
            writeln!(f, "Proxy: {}", proxy)?;
        }
        write!(f, "Provider: {}", self.provider)?;

        Ok(())
    }
}
