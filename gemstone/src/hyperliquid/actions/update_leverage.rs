use super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperUpdateLeverage {
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    pub asset: u32,
    #[serde(rename = "isCross")]
    pub is_cross: bool,
    pub leverage: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub r#type: String,
}

impl HyperUpdateLeverage {
    pub fn new(asset: u32, is_cross: bool, leverage: u64) -> Self {
        Self {
            hyperliquid_chain: MAINNET.to_string(),
            asset,
            is_cross,
            leverage,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            r#type: "updateLeverage".to_string(),
        }
    }
}
