use google_calendar::Client;

mod config;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = config::Configuration::load()?;

    Client

    // let google_calendar =
    //     Client::new_from_env(String::from("token"), String::from("refresh-token")).await;

    // let events = google_calendar
    //     .events()
    //     .list(
    //         "primary",
    //         "",
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //         Default::default(),
    //     )
    //     .await?;

    eprintln!("{config:#?}");

    Ok(())
}
