use crate::twitch::TwitchApi;
use log::{error, info};
use std::env;

mod twitch;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Env file is not loaded into the project");
    // env_logger::init();
    // let mut twitch_client = TwitchApi::new().expect("Twitch client incorrectly configured");
    // twitch_client
    //     .get_and_store_token()
    //     .await
    //     .expect("Error getting access token");
    // let streamer = env::var("STREAMER").unwrap();
    // let stream_info = twitch_client
    //     .get_stream_info(&streamer)
    //     .await
    //     .expect("Error getting stream info");
    // match stream_info {
    //     Some(info) => info!("Streamer is online with {} viewers", info.viewer_count),
    //     None => {
    //         error!("Streamer is offline {}", streamer);
    //     }
    // }
    TwitchApi::listen_to_chat().await;
}
