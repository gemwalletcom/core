use crate::core::actions::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(Clone, serde::Serialize)]
pub struct TokenDelegate {
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

impl TokenDelegate {
    pub fn new(validator: String, wei: u64, is_undelegate: bool, nonce: u64) -> Self {
        Self {
            validator: validator.to_lowercase(),
            wei,
            is_undelegate,
            nonce,
            r#type: "tokenDelegate".to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
