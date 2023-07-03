//use google_calendar::Client;

mod config;

use serde::{Deserialize};

#[derive(Debug, Deserialize)]
struct EventsResponseData {
    summary: String,
    description: String,
    items: Vec<Event>,
}

#[derive(Debug, Deserialize)]
struct Event {
    summary: String,
    description: Option<String>,
    location: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = config::Configuration::load()?;

    let endpoint = format!("https://www.googleapis.com/calendar/v3/calendars/{}/events?key={}",config.google_calendar_id, config.google_api_key);

    let response = reqwest::get(endpoint).await?;

    if response.status().is_success() {
        let json_body = response.text().await?;
        let data: EventsResponseData = serde_json::from_str(&json_body)?;
      
        eprintln!("{data:#?}");

    } else {
        println!("Request failed with status code: {}", response.status());
    }

    //eprintln!("{config:#?}");

    Ok(())
}
