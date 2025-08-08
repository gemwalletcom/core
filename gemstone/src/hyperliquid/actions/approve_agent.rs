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
    pub r#type: String,
}

impl HyperApproveAgent {
    pub fn new(agent_address: String, agent_name: String, nonce: u64) -> Self {
        Self {
            agent_address: agent_address.to_lowercase(),
            agent_name,
            hyperliquid_chain: MAINNET.to_string(),
            nonce,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            r#type: "approveAgent".to_string(),
        }
    }
}
