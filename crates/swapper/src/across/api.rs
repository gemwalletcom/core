use crate::{
    SwapperError,
    alien::{RpcProvider, Target},
    client_factory::create_eth_client,
};
use primitives::{Chain, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const FUNDS_DEPOSITED_TOPIC: &str = "0x32ed1a409ef04c7b0227189c3a103dc5ac10e775a15b785dcc510201f7c25ad3";

#[derive(Debug, Clone)]
pub struct AcrossApi {
    pub url: String,
    pub provider: Arc<dyn RpcProvider>,
}

impl AcrossApi {
    pub fn new(provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            url: "https://app.across.to".into(),
            provider,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositStatus {
    pub status: String,
    pub deposit_id: String,
    pub deposit_tx_hash: String,
    pub fill_tx: Option<String>,
    pub destination_chain_id: u64,
    pub deposit_refund_tx_hash: Option<String>,
}

impl DepositStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status.as_str() {
            "filled" => SwapStatus::Completed,
            "refunded" => SwapStatus::Refunded,
            _ => SwapStatus::Pending,
        }
    }
}

impl AcrossApi {
    pub async fn deposit_status(&self, chain: Chain, tx_hash: &str) -> Result<DepositStatus, SwapperError> {
        let receipt = create_eth_client(self.provider.clone(), chain)?
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(SwapperError::from)?;

        let deposit_log = receipt
            .logs
            .iter()
            .find(|log| {
                log.topics
                    .first()
                    .map(|topic| topic.eq_ignore_ascii_case(FUNDS_DEPOSITED_TOPIC))
                    .unwrap_or(false)
            })
            .ok_or_else(|| SwapperError::NetworkError("FundsDeposited event not found".into()))?;

        if deposit_log.topics.len() < 3 {
            return Err(SwapperError::NetworkError("invalid FundsDeposited topics".into()));
        }
        // The deposit ID is in topics[2] (topics[0] is event signature, topics[1] is destination chain ID)
        let deposit_id_hex = deposit_log.topics[2].clone();

        // Convert hex deposit ID to decimal string
        let deposit_id = if let Some(stripped) = deposit_id_hex.strip_prefix("0x") {
            u64::from_str_radix(stripped, 16)
                .map_err(|e| SwapperError::NetworkError(format!("Failed to parse deposit ID: {}", e)))?
                .to_string()
        } else {
            deposit_id_hex
        };

        let url = format!("{}/api/deposit/status?originChainId={}&depositId={}", self.url, chain.network_id(), &deposit_id);
        let target = Target::get(&url);
        let response = self.provider.request(target).await?;
        let status: DepositStatus = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(status)
    }
}
