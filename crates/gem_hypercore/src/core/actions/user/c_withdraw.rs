use crate::core::actions::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(Clone, serde::Serialize)]
pub struct CWithdraw {
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub r#type: String,
    pub wei: u64,
}

impl CWithdraw {
    pub fn new(wei: u64, nonce: u64) -> Self {
        Self {
            hyperliquid_chain: MAINNET.to_string(),
            nonce,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            r#type: "cWithdraw".to_string(),
            wei,
        }
    }
}
