use super::super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(Clone, serde::Serialize)]
pub struct UsdClassTransfer {
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

impl UsdClassTransfer {
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
