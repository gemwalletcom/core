use super::{MAINNET, SIGNATURE_CHAIN_ID};

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperApproveAgent {
    #[serde(rename = "agentAddress")]
    pub agent_address: String,
    #[serde(rename = "agentName")]
    pub agent_name: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "type")]
    pub action_type: String,
}

impl HyperApproveAgent {
    pub fn new(agent_address: String, agent_name: String, nonce: u64) -> Self {
        Self {
            action_type: "approveAgent".to_string(),
            hyperliquid_chain: MAINNET.to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            agent_address,
            agent_name,
            nonce,
        }
    }
}
