use crate::error::Result;
use crate::LookupResponse;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupCache {
    pub response: LookupResponse,
    pub response_time: SystemTime,
}

impl LookupCache {
    #[cfg(unix)]
    const CACHE_PATH: &'static str = "/private/tmp/public-ip-cache.txt";
    #[cfg(not(unix))]
    const CACHE_PATH: &'static str = "public-ip-cache.txt";

    pub fn new(response: LookupResponse) -> LookupCache {
        LookupCache {
            response,
            response_time: SystemTime::now(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string(self)?;
        let mut file = File::create(Self::CACHE_PATH)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load() -> Result<LookupCache> {
        let mut file = File::open(Self::CACHE_PATH)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let deserialized: LookupCache = serde_json::from_str(&contents)?;
        Ok(deserialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let response = LookupResponse::new("1.1.1.1".to_string());
        let cache = LookupCache::new(response);
        cache.save().unwrap();
        let cached = LookupCache::load().unwrap();
        assert_eq!(cached.response.ip, "1.1.1.1", "IP address not matching");
    }
}
