//! Lookup response.

use crate::lookup::LookupProvider;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lookup response containing information like IP, country, city, hostname etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LookupResponse {
    /// Public IP address.
    pub ip: String,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub time_zone: Option<String>,
    /// Autonomous System Number.
    pub asn: Option<String>,
    /// Autonomous System Organization.
    pub asn_org: Option<String>,
    /// Hostname of the IP address.
    pub hostname: Option<String>,
    /// Is the IP a proxy?
    pub proxy: Option<bool>,
    /// Provider that was used for the lookup.
    pub provider: LookupProvider,
}

impl LookupResponse {
    /// Create a new lookup response.
    pub fn new(ip: String, provider: LookupProvider) -> Self {
        LookupResponse {
            ip,
            continent: None,
            country: None,
            country_code: None,
            region: None,
            region_code: None,
            postal_code: None,
            city: None,
            latitude: None,
            longitude: None,
            time_zone: None,
            asn: None,
            asn_org: None,
            hostname: None,
            proxy: None,
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
            write!(f, "Region: {}", region)?;
        }
        if let Some(region_code) = &self.region_code {
            writeln!(f, " ({})", region_code)?;
        } else {
            writeln!(f)?;
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
        if let Some(proxy) = &self.proxy {
            writeln!(f, "Proxy: {}", proxy)?;
        }
        write!(f, "Provider: {}", self.provider)?;

        Ok(())
    }
}
