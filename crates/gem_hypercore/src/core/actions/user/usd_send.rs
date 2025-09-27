use crate::core::actions::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(Clone, serde::Serialize)]
pub struct UsdSend {
    pub destination: String,
    pub amount: String,
    pub time: u64,
    pub r#type: String,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
}

impl UsdSend {
    pub fn new(amount: String, destination: String, time: u64) -> Self {
        Self {
            destination: destination.to_lowercase(),
            amount,
            time,
            r#type: "usdSend".to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
