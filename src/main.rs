use std::sync::Arc;
use unterstutzen::Calendar;
use unterstutzen::Configuration;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Arc::new(Configuration::load()?);
    let calendar = Calendar::from(&config);
    let events = calendar.events().await?;
    eprintln!("{events:#?}");
    Ok(())
}
