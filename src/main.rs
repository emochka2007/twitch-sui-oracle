use crate::twitch::TwitchApi;
mod pg;
use crate::pg::pg::{PgClient, PgConnect};
use std::env;

mod sui;
mod twitch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Env file is not loaded into the project");

    let coin = sui::start_client().await?;
    return Ok(());
    // postgres migration
    let pool = PgConnect::create_pool_from_env()?;
    let client = pool.get().await?;
    PgConnect::run_migrations(&client).await?;

    let mut twitch_client = TwitchApi::new().expect("Twitch client incorrectly configured");
    twitch_client
        .get_and_store_token()
        .await
        .expect("Error getting access token");

    let streamer = env::var("STREAMER").unwrap();
    let stream_info = twitch_client
        .get_stream_info(&streamer)
        .await
        .expect("Error getting stream info");
    tracing::info!("Streamer is online with viewers");
    match stream_info {
        Some(info) => {
            tracing::info!("Streamer is online with {} viewers", info.viewer_count);
            TwitchApi::listen_to_chat().await?;
        }
        None => {
            tracing::error!("Streamer is offline {}", streamer);
        }
    }
    Ok(())
}
