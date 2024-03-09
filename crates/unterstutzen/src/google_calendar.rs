use std::sync::Arc;

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::config::Configuration;

pub struct GoogleCalendar {
    config: Arc<Configuration>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Events {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub items: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub creator: Creator,
    pub start: Option<Start>,
    pub end: Option<End>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Start {
    pub date: Option<String>,
    pub date_time: Option<String>,
    pub time_zone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct End {
    pub date: Option<String>,
    pub date_time: Option<String>,
    pub time_zone: Option<String>,
}

impl From<&Arc<Configuration>> for GoogleCalendar {
    fn from(config: &Arc<Configuration>) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl GoogleCalendar {
    pub async fn events(&self) -> anyhow::Result<Events> {
        // Limit the time range for a year for the moment to see future events for the next year,
        // with a max result count of 500 for this result page
        // TODO: better query and filtering
        let start_date = Utc::now();
        let start_date_formatted = start_date.to_rfc3339();
        let time_min = encode(&start_date_formatted);

        let end_date = start_date
            .with_year(start_date.year() + 1)
            .expect("Failed to add one year");
        let end_date_formatted = end_date.to_rfc3339();
        let time_max = encode(&end_date_formatted);

        // TODO: singleEvents returns a bunch of isntances with the startDate of the original instance,
        // needs to be updated to use originalStartDate for recurrences (has recurringEventId)

        let endpoint = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events?key={}&singleEvents=true&orderby=starttime&timeMin={}&timeMax={}&maxResults=500",
            self.config.google_calendar_id, self.config.google_api_key, time_min, time_max
        );

        //tracing::info!("{}", endpoint);

        let response = reqwest::get(endpoint).await?;

        if !response.status().is_success() {
            //anyhow::bail!("accesing calendar data failed: {response:?}");
            panic!("accesing calendar data failed: {response:?}");
        }

        let json_body = response.text().await?;
        //tracing::info!("{}", json_body);
        Ok(serde_json::from_str(&json_body)?)
    }
}
