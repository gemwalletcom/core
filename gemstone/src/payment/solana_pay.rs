use crate::{
    network::{AlienProvider, AlienTarget},
    GemstoneError,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, uniffi::Object)]
pub struct SolanaPay {
    pub provider: Arc<dyn AlienProvider>,
}

#[derive(Debug, Deserialize, uniffi::Record)]
pub struct SolanaPayLabel {
    pub label: String,
    #[serde(rename = "icon")]
    pub icon_url: String,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, uniffi::Record)]
pub struct SolanaPayTransaction {
    pub message: Option<String>,
    pub transaction: Option<String>, // base64
    pub error: Option<String>,
}

#[uniffi::export]
impl SolanaPay {
    #[uniffi::constructor]
    fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    async fn get_label(&self, link: &str) -> Result<SolanaPayLabel, GemstoneError> {
        let target = AlienTarget::get(link);
        let response = self.provider.request(target).await?;
        let label = serde_json::from_slice::<SolanaPayLabel>(&response).map_err(|_| GemstoneError::AnyError {
            msg: "Failed to get solana pay label and icon".into(),
        })?;
        if let Some(error) = label.error {
            return Err(GemstoneError::AnyError { msg: error });
        }
        Ok(label)
    }

    async fn post_account(&self, link: &str, account: &str) -> Result<SolanaPayTransaction, GemstoneError> {
        let body = serde_json::json!({
            "account": account,
        });
        let target = AlienTarget::post_json(link, body);
        let response = self.provider.request(target).await?;
        let transaction = serde_json::from_slice::<SolanaPayTransaction>(&response).map_err(|_| GemstoneError::AnyError {
            msg: "Failed to get solana pay transaction".into(),
        })?;
        if let Some(error) = transaction.error {
            return Err(GemstoneError::AnyError { msg: error });
        }
        Ok(transaction)
    }
}
