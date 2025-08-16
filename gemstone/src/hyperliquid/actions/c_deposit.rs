use super::{HYPERCORE_SIGNATURE_CHAIN_ID, MAINNET};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperCDeposit {
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub r#type: String,
    pub wei: u64,
}

impl HyperCDeposit {
    pub fn new(wei: u64, nonce: u64) -> Self {
        Self {
            hyperliquid_chain: MAINNET.to_string(),
            nonce,
            signature_chain_id: HYPERCORE_SIGNATURE_CHAIN_ID.to_string(),
            r#type: "cDeposit".to_string(),
            wei,
        }
    }
}
