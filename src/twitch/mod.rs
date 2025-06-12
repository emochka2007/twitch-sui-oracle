use log::info;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::env::VarError;
use std::error::Error;
use std::fmt::Debug;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
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

pub struct ChatMessage {
    message: String,
    username: String,
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

    pub async fn connect_to_irc(&self) -> Result<(), Box<dyn Error>> {
        let stream = TcpStream::connect("irc.chat.twitch.tv:6667").await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader).lines();

        let oauth_token = self.access_token.as_ref().unwrap();
        let username = "";
        let channel = "your_channel"; // without the #

        writer
            .write_all(format!("PASS {}\r\n", oauth_token).as_bytes())
            .await?;
        writer
            .write_all(format!("NICK {}\r\n", username).as_bytes())
            .await?;
        writer
            .write_all(format!("JOIN #{}\r\n", channel).as_bytes())
            .await?;

        println!("Joined #{} chat", channel);

        while let Some(line) = reader.next_line().await? {
            println!("> {}", line);
        }
        Ok(())
    }

    pub async fn exchange_code_for_token(&self) -> Result<String, reqwest::Error> {
        let client = Client::new();
        let code = env::var("TWITCH_CODE").expect("twitch code is not presented");
        let redirect_uri =
            env::var("TWITCH_REDIRECT_URI").expect("TWITCH_REDIRECT_URI is not presented");
        let params = [
            ("client_id", &self.client),
            ("client_secret", &self.secret),
            ("code", &code),
            ("grant_type", &"authorization_code".to_string()),
            ("redirect_uri", &redirect_uri),
        ];

        let res = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        Ok(res)
    }

    pub async fn listen_to_chat() {
        tracing_subscriber::fmt::init();
        // default configuration is to join chat as anonymous.
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        // first thing you should do: start consuming incoming messages,
        // otherwise they will back up.
        let join_handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let Privmsg(priv_msg) = message {
                    tracing::info!("Received message: {:?}", priv_msg.message_text);
                    tracing::info!("Received message: {:?}", priv_msg.sender.login);
                }
            }
        });

        // join a channel
        // This function only returns an error if the passed channel login name is malformed,
        // so in this simple case where the channel name is hardcoded we can ignore the potential
        // error with `unwrap`.
        client.join("rostislav_999".to_owned()).unwrap();

        // keep the tokio executor alive.
        // If you return instead of waiting the background task will exit.
        join_handle.await.unwrap();
    }
}
