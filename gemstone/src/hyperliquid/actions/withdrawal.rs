use super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperWithdrawalRequest {
    pub amount: String,
    pub destination: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub time: u64,
    pub r#type: String,
}

impl HyperWithdrawalRequest {
    pub fn new(amount: String, time: u64, destination: String) -> Self {
        Self {
            amount,
            destination: destination.to_lowercase(),
            hyperliquid_chain: MAINNET.to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            time,
            r#type: "withdraw3".to_string(),
        }
    }
}
