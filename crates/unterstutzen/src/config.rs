use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub google_api_key: String,
    pub google_calendar_id: String,
}

impl Configuration {
    pub fn new(google_api_key: String, google_calendar_id: String) -> Self {
        Self {
            google_api_key,
            google_calendar_id,
        }
    }

    pub fn from_env() -> Result<Self, crate::Error> {
        let google_api_key = std::env::var("GOOGLE_API_KEY")?;
        let google_calendar_id = std::env::var("GOOGLE_CALENDAR_ID")?;

        Ok(Self {
            google_api_key,
            google_calendar_id,
        })
    }

    pub fn load() -> Result<Self, crate::Error> {
        let path = PathBuf::from("americano.toml");
        Self::from_toml_in_file(&path)
    }

    pub fn from_toml_in_file(path: &Path) -> Result<Self, crate::Error> {
        let text = std::fs::read_to_string(path)?;
        Self::from_toml_str(&text)
    }

    pub fn from_toml_str(text: &str) -> Result<Self, crate::Error> {
        Ok(toml::from_str(text)?)
    }
}
