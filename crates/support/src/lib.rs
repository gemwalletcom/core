mod bot_client;
mod chatwoot_client;
mod client;
mod model;

pub use bot_client::SupportBotClient;
pub use chatwoot_client::{ChatwootApiClient, ChatwootConversationMessage};
pub use claude::{ClaudeClient, Message as ClaudeMessage};
pub use client::SupportClient;
pub use model::*;
