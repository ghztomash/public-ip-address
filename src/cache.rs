use crate::{error::Result, LookupResponse};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, fs::File, io::prelude::*, time::SystemTime};

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseCache {
    pub response: LookupResponse,
    pub response_time: SystemTime,
}

impl ResponseCache {
    #[cfg(unix)]
    const CACHE_PATH: &'static str = "/private/tmp/public-ip-cache";
    #[cfg(not(unix))]
    const CACHE_PATH: &'static str = "public-ip-cache";

    pub fn new(response: LookupResponse) -> ResponseCache {
        ResponseCache {
            response,
            response_time: SystemTime::now(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string(self)?;
        let encoded = BASE64_STANDARD.encode(serialized);
        let mut file = File::create(Self::CACHE_PATH)?;
        file.write_all(encoded.as_bytes())?;
        Ok(())
    }

    pub fn load() -> Result<ResponseCache> {
        let mut file = File::open(Self::CACHE_PATH)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let decoded = String::from_utf8(BASE64_STANDARD.decode(contents)?)?;
        let deserialized: ResponseCache = serde_json::from_str(&decoded)?;
        Ok(deserialized)
    }

    pub fn delete() -> Result<()> {
        fs::remove_file(Self::CACHE_PATH)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::LookupProvider;

    #[test]
    fn test_cache() {
        let response = LookupResponse::new(
            "1.1.1.1".to_string(),
            LookupProvider::Mock("1.1.1.1".to_string()),
        );
        let cache = ResponseCache::new(response);
        cache.save().unwrap();
        let cached = ResponseCache::load().unwrap();
        assert_eq!(cached.response.ip, "1.1.1.1", "IP address not matching");
    }
}
