use std::error::Error;

use gem_tracing::info_with_fields;
use primitives::ConfigKey;
use storage::ConfigCacher;

use crate::chatwoot_client::{ChatwootApiClient, ChatwootConversationMessage};
use crate::model::{ChatwootWebhookPayload, MessageType};
use claude::{ClaudeClient, Message};

const HANDOFF_MARKER: &str = "[HANDOFF]";

pub struct SupportBotClient {
    chatwoot_client: ChatwootApiClient,
    claude_client: ClaudeClient,
    config: ConfigCacher,
}

impl SupportBotClient {
    pub fn new(chatwoot_client: ChatwootApiClient, claude_client: ClaudeClient, config: ConfigCacher) -> Self {
        Self {
            chatwoot_client,
            claude_client,
            config,
        }
    }

    fn load_config(&self) -> Result<BotConfig, Box<dyn Error + Send + Sync>> {
        Ok(BotConfig {
            system_prompt: self.config.get(ConfigKey::SupportBotSystemPrompt)?,
            model: self.config.get(ConfigKey::SupportBotModel)?,
            version: self.config.get(ConfigKey::SupportBotApiVersion)?,
            max_tokens: self.config.get_i32(ConfigKey::SupportBotMaxTokens)? as u32,
        })
    }

    pub async fn process_incoming(&self, webhook: &ChatwootWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let account_id = match webhook.get_account_id() {
            Some(id) => id,
            None => {
                info_with_fields!("bot webhook missing account_id", event = webhook.event);
                return Ok(true);
            }
        };
        let conversation_id = match webhook.get_conversation_id() {
            Some(id) => id,
            None => {
                info_with_fields!("bot webhook missing conversation_id", event = webhook.event);
                return Ok(true);
            }
        };

        let config = self.load_config()?;
        let conversation_messages = self.chatwoot_client.get_conversation_messages(account_id, conversation_id).await?;

        let messages: Vec<Message> = conversation_messages.iter().filter_map(|msg| msg.as_claude_message()).collect();

        if messages.is_empty() {
            return Ok(true);
        }

        let response = self
            .claude_client
            .generate_response(&config.system_prompt, messages, &config.model, &config.version, config.max_tokens)
            .await?;

        if response.contains(HANDOFF_MARKER) {
            let clean_response = response.replace(HANDOFF_MARKER, "").trim().to_string();
            if !clean_response.is_empty() {
                self.chatwoot_client.send_message(account_id, conversation_id, &clean_response).await?;
            }
            self.chatwoot_client.toggle_conversation_status(account_id, conversation_id, "open").await?;
            info_with_fields!("bot handoff to human", account_id = account_id, conversation_id = conversation_id);
        } else {
            self.chatwoot_client.send_message(account_id, conversation_id, &response).await?;
        }

        info_with_fields!("bot response sent", account_id = account_id, conversation_id = conversation_id);
        Ok(true)
    }
}

struct BotConfig {
    system_prompt: String,
    model: String,
    version: String,
    max_tokens: u32,
}

impl ChatwootConversationMessage {
    fn as_claude_message(&self) -> Option<Message> {
        let content = self.content.as_deref()?.to_string();
        if content.is_empty() {
            return None;
        }
        let role = match self.message_type {
            MessageType::Incoming => "user",
            MessageType::Outgoing => "assistant",
        };
        Some(Message { role: role.to_string(), content })
    }
}
