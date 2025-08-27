use crate::core::actions::{HYPERCORE_SIGNATURE_CHAIN_ID, MAINNET};

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
            signature_chain_id: HYPERCORE_SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
