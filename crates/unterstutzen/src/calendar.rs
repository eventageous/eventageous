use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::Configuration;

pub struct Calendar {
    config: Arc<Configuration>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Events {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub items: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
}

impl From<&Arc<Configuration>> for Calendar {
    fn from(config: &Arc<Configuration>) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl Calendar {
    pub async fn events(&self) -> Result<Events, crate::Error> {
        let endpoint = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events?key={}",
            self.config.google_calendar_id, self.config.google_api_key
        );

        let response = reqwest::get(endpoint).await?;

        if !response.status().is_success() {
            //anyhow::bail!("accesing calendar data failed: {response:?}");
            panic!("accesing calendar data failed: {response:?}");
        }

        let json_body = response.text().await?;
        Ok(serde_json::from_str(&json_body)?)
    }
}
