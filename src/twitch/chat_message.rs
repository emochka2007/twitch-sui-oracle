use crate::pg::pg::PgConnect;
use regex::Regex;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use tracing::info;

type GlobalError = Box<dyn Error>;
#[derive(Debug)]
pub struct ChatMessage {
    command: ChatCommands,
    user_id: i64,
}

#[derive(Debug)]
enum ChatCommands {
    STORE_CHAT_MESSAGE(String),
    CLAIM_NFT(String),
    Unknown(String),
}
impl Display for ChatCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ChatCommands::STORE_CHAT_MESSAGE(s) => "!STORE".to_string(),
            ChatCommands::CLAIM_NFT(s) => "!NFT".to_string(),
            ChatCommands::Unknown(s) => "UNKNOWN".to_string(),
        };
        write!(f, "{}", str)
    }
}
impl FromStr for ChatCommands {
    type Err = GlobalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg = Regex::new(r"^(![A-Z]+)\s(.*)")?;
        let captures = reg.captures(s).ok_or("Failed to capture")?;
        // todo clean up unwraps
        let command = captures.get(1).ok_or("Failed to capture command")?;
        let text = captures
            .get(2)
            .ok_or("Failed to capture text")?
            .as_str()
            .to_string();
        let parsed_command = match command.as_str() {
            "!STORE" => ChatCommands::STORE_CHAT_MESSAGE(text),
            "!NFT" => ChatCommands::CLAIM_NFT(text),
            _ => ChatCommands::Unknown(text),
        };
        Ok(parsed_command)
    }
}
impl ChatMessage {
    pub fn new(full_message: String, user_id: i64) -> Self {
        let command = Self::parse(full_message).unwrap();

        Self { command, user_id }
    }

    fn parse(full_message: String) -> Result<ChatCommands, GlobalError> {
        ChatCommands::from_str(&full_message)
    }

    pub async fn verify_and_send(&self) -> Result<(), GlobalError> {
        // let command = self.parse().unwrap_or(ChatCommands::Unknown);
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        match &self.command {
            ChatCommands::STORE_CHAT_MESSAGE(text) => {
                let query =
                    "INSERT INTO chat_messages ( user_id, text, command ) VALUES ($1, $2, $3)";
                client
                    .query(query, &[&self.user_id, text, &self.command.to_string()])
                    .await?;
            }
            ChatCommands::CLAIM_NFT(text) => {
                let query =
                    "INSERT INTO chat_messages (user_id, text, command) VALUES ($1, $2, $3)";
                client
                    .query(query, &[&self.user_id, text, &self.command.to_string()])
                    .await?;
            }
            ChatCommands::Unknown(text) => {
                info!("Skipping message {}", text);
            }
        };

        Ok(())
    }
}
