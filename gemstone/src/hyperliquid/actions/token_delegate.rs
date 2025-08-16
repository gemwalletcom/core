use super::{HYPERCORE_SIGNATURE_CHAIN_ID, MAINNET};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperTokenDelegate {
    pub validator: String,
    pub wei: u64,
    #[serde(rename = "isUndelegate")]
    pub is_undelegate: bool,
    pub nonce: u64,
    pub r#type: String,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
}

impl HyperTokenDelegate {
    pub fn new(validator: String, wei: u64, is_undelegate: bool, nonce: u64) -> Self {
        Self {
            validator: validator.to_lowercase(),
            wei,
            is_undelegate,
            nonce,
            r#type: "tokenDelegate".to_string(),
            signature_chain_id: HYPERCORE_SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
