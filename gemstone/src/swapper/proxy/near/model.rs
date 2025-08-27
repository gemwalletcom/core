use primitives::{swap::SwapStatus, Chain};
use serde::{Deserialize, Serialize};

use super::parse_near_asset_chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsTransactionResult {
    pub status: NearIntentsTransactionStatus,
    pub swap_details: NearIntentsSwapDetails,
    pub quote_response: NearIntentsQuoteResponse,
}

impl NearIntentsTransactionResult {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status {
            NearIntentsTransactionStatus::Success => SwapStatus::Completed,
            NearIntentsTransactionStatus::Refunded => SwapStatus::Refunded,
            NearIntentsTransactionStatus::Pending => SwapStatus::Pending,
            NearIntentsTransactionStatus::Failed => SwapStatus::Failed,
        }
    }

    pub fn to_chain(&self) -> Option<Chain> {
        parse_near_asset_chain(&self.quote_response.quote_request.destination_asset)
    }

    pub fn to_tx_hash(&self) -> Option<String> {
        self.swap_details.destination_chain_tx_hashes.first().map(|chain_hash| chain_hash.hash.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NearIntentsTransactionStatus {
    Success,
    Pending,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsSwapDetails {
    pub destination_chain_tx_hashes: Vec<NearIntentsChainHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NearIntentsChainHash {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsQuoteResponse {
    pub quote_request: NearIntentsQuoteRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NearIntentsQuoteRequest {
    pub origin_asset: String,
    pub destination_asset: String,
}
