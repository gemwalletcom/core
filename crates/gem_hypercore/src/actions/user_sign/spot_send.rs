use super::super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(Clone, serde::Serialize)]
pub struct SpotSend {
    pub destination: String,
    pub amount: String,
    pub token: String,
    pub time: u64,
    pub r#type: String,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
}

impl SpotSend {
    pub fn new(amount: String, destination: String, time: u64, token: String) -> Self {
        Self {
            destination: destination.to_lowercase(),
            amount,
            token,
            time,
            r#type: "spotSend".to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
