use super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperWithdrawalRequest {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub amount: String,
    pub time: u64,
    pub destination: String,
}

impl HyperWithdrawalRequest {
    pub fn new(amount: String, time: u64, destination: String) -> Self {
        Self {
            action_type: "withdraw3".to_string(),
            hyperliquid_chain: MAINNET.to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            amount,
            time,
            destination,
        }
    }
}
