use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::calendar_transformer;
use crate::calendar_transformer::calendar_transformer::google_to_americano;
use crate::config::Configuration;
use crate::google_calendar::GoogleCalendar;

pub struct Calendar {
    config: Arc<Configuration>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AmericanoEvents {
    pub events: Vec<AmericanoEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmericanoEvent {
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub creator_email: String,
    pub creator_name: String,
    pub start_datetime: String,
    pub start_timezone: String,
    pub end_datetime: String,
    pub end_timezone: String,
}

impl From<&Arc<Configuration>> for Calendar {
    fn from(config: &Arc<Configuration>) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl Calendar {
    pub async fn events(&self) -> anyhow::Result<AmericanoEvents> {
        // Using Google Calendar API behind the scenes

        let google_calendar = GoogleCalendar::from(&self.config.clone());
        let g_events = google_calendar.events().await.unwrap();

        // Transform events to Americano event format
        let americano_events = google_to_americano(g_events);
        Ok(americano_events)
    }
}
