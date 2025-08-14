use super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperUsdClassTransfer {
    pub r#type: String,
    pub amount: String,
    #[serde(rename = "toPerp")]
    pub to_perp: bool,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
}

impl HyperUsdClassTransfer {
    pub fn new(amount: String, to_perp: bool, nonce: u64) -> Self {
        Self {
            r#type: "usdClassTransfer".to_string(),
            amount,
            to_perp,
            nonce,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
