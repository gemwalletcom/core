use super::super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(Clone, serde::Serialize)]
pub struct ApproveBuilderFee {
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    #[serde(rename = "maxFeeRate")]
    pub max_fee_rate: String, // percent string 0.001%
    pub builder: String,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub r#type: String,
}

impl ApproveBuilderFee {
    pub fn new(max_fee_rate: String, builder: String, nonce: u64) -> Self {
        Self {
            hyperliquid_chain: MAINNET.to_string(),
            max_fee_rate,
            builder: builder.to_lowercase(),
            nonce,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            r#type: "approveBuilderFee".to_string(),
        }
    }
}
