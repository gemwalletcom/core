use super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperApproveBuilderFee {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "maxFeeRate")]
    pub max_fee_rate: String, // percent string 0.001%
    pub builder: String,
    pub nonce: u64,
}

impl HyperApproveBuilderFee {
    pub fn new(max_fee_rate: String, builder: String, nonce: u64) -> Self {
        Self {
            action_type: "approveBuilderFee".to_string(),
            hyperliquid_chain: MAINNET.to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            max_fee_rate,
            builder: builder.to_lowercase(),
            nonce,
        }
    }
}
