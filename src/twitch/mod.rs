use reqwest::Client;
pub mod chat_message;
use crate::twitch::chat_message::ChatMessage;
use serde::Deserialize;
use std::env;
use std::env::VarError;
use std::error::Error;
use std::fmt::Debug;
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage::Privmsg;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

pub struct TwitchApi {
    secret: String,
    client: String,
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TwitchStreamResponse {
    pub data: Vec<StreamInfo>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamInfo {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_id: String,
    pub game_name: String,
    #[serde(rename = "type")]
    pub stream_type: String,
    pub title: String,
    pub viewer_count: u32,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String,
    pub tag_ids: Vec<String>,
    pub tags: Vec<String>,
    pub is_mature: bool,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i32,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u32,
    scope: Vec<String>,
    token_type: String,
}

impl TwitchApi {
    pub fn new() -> Result<Self, VarError> {
        let secret = env::var("TWITCH_SECRET")?;
        let client = env::var("TWITCH_CLIENT")?;
        Ok(Self {
            secret,
            client,
            access_token: None,
        })
    }

    pub async fn get_and_store_token(&mut self) -> Result<(), reqwest::Error> {
        let form_data = [
            ("client_id", &self.client),
            ("client_secret", &self.secret),
            ("grant_type", &"client_credentials".to_string()),
        ];
        let client = Client::new();
        match client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&form_data)
            .send()
            .await
        {
            Ok(response) => {
                let parsed_response = response.json::<TokenResponse>().await?;
                self.access_token = Some(parsed_response.access_token);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub async fn get_stream_info(
        &self,
        user_login: &str,
    ) -> Result<Option<StreamInfo>, reqwest::Error> {
        let url = format!(
            "https://api.twitch.tv/helix/streams?user_login={}",
            user_login
        );
        let client = Client::new();
        let access_token = self
            .access_token
            .as_ref()
            .expect("Access token must be presented");
        let bearer_token = format!("Bearer {}", access_token);
        let request = client
            .get(&url)
            .header("Client-ID", &self.client)
            .header("Authorization", bearer_token);
        match request.send().await {
            Ok(response) => {
                let parsed: TwitchStreamResponse =
                    serde_json::from_str(&response.text().await?).unwrap();
                if let Some(item) = parsed.data.first() {
                    Ok(Some(item.clone()))
                } else {
                    Ok(None)
                }
            }
            Err(error) => Err(error),
        }
    }

    pub async fn listen_to_chat() -> Result<(), Box<dyn Error>> {
        // default configuration is to join chat as anonymous.
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        // first thing you should do: start consuming incoming messages,
        // otherwise they will back up.
        let join_handle: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> =
            tokio::spawn(async move {
                while let Some(message) = incoming_messages.recv().await {
                    if let Privmsg(priv_msg) = message {
                        let msg_id_tag = priv_msg.source.tags.0.get("msg-id");
                        match msg_id_tag {
                            Some(_) => {
                                let chat_message = ChatMessage::new(
                                    priv_msg.message_text,
                                    priv_msg.sender.id.parse()?,
                                );
                                chat_message.verify_and_send().await.unwrap();
                            }
                            None => (),
                        }
                    }
                }
                Ok(())
            });

        // join a channel
        // This function only returns an error if the passed channel login name is malformed,
        // so in this simple case where the channel name is hardcoded we can ignore the potential
        // error with `unwrap`.
        let streamer_channel = env::var("STREAMER").expect("STREAMER env is not set");
        client.join(streamer_channel.to_owned()).unwrap();

        // keep the tokio executor alive.
        // If you return instead of waiting the background task will exit.
        join_handle.await?.expect("Error in join handle");
        Ok(())
    }
}
