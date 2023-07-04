//use google_calendar::Client;

mod calendar;
mod config;

use std::sync::Arc;

use calendar::Calendar;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Arc::new(config::Configuration::load()?);
    let calendar = Calendar::from(&config);
    let events = calendar.events().await?;
    eprintln!("{events:#?}");
    Ok(())
}
