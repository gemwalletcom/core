use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LayerZeroScanApi {
    pub url: String,
    pub provider: Arc<dyn AlienProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub data: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub status: MessageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageStatusName {
    Inflight,
    Confirming,
    Failed,
    Delivered,
    Blocked,
    PayloadStored,
    ApplicationBurned,
    ApplicationSkipped,
    UnresolvableCommand,
    MalformedCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatus {
    pub name: MessageStatusName,
    pub message: Option<String>,
}

#[allow(dead_code)]
impl MessageStatus {
    pub fn is_delivered(&self) -> bool {
        matches!(self.name, MessageStatusName::Delivered)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.name, MessageStatusName::Failed)
    }

    pub fn is_pending(&self) -> bool {
        matches!(
            self.name,
            MessageStatusName::Inflight | MessageStatusName::Confirming | MessageStatusName::PayloadStored
        )
    }
}

impl LayerZeroScanApi {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            url: "https://scan.layerzero-api.com/v1".into(),
            provider,
        }
    }

    pub async fn get_message_by_tx(&self, tx_hash: &str) -> Result<MessageResponse, SwapperError> {
        let url = format!("{}/messages/tx/{}", self.url, tx_hash);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_tx() {
        let message_tx = include_str!("mock/message_tx.json");
        let message_tx: MessageResponse = serde_json::from_str(message_tx).unwrap();
        assert_eq!(message_tx.data.len(), 1);

        let message = &message_tx.data[0];
        assert!(message.status.is_delivered());
    }

    #[test]
    fn test_message_status() {
        let status = MessageStatus {
            name: MessageStatusName::Delivered,
            message: None,
        };
        assert!(status.is_delivered());
        assert!(!status.is_failed());
        assert!(!status.is_pending());

        let status = MessageStatus {
            name: MessageStatusName::Failed,
            message: None,
        };
        assert!(!status.is_delivered());
        assert!(status.is_failed());
        assert!(!status.is_pending());

        let status = MessageStatus {
            name: MessageStatusName::Inflight,
            message: None,
        };
        assert!(!status.is_delivered());
        assert!(!status.is_failed());
        assert!(status.is_pending());
    }
}
