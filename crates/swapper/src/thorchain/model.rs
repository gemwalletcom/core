use num_bigint::BigInt;
use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

use super::{asset::THORChainAsset, chain::THORChainName, constants::THORCHAIN_INBOUND_ADDRESS};
use crate::SwapperError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapRequest {
    pub from_asset: String,
    pub to_asset: String,
    pub amount: String,
    pub affiliate: String,
    pub affiliate_bps: i64,
    pub streaming_interval: i64,
    pub streaming_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSwapResponse {
    pub expected_amount_out: String,
    pub inbound_address: Option<String>,
    pub router: Option<String>,
    pub fees: QuoteFees,
    pub total_swap_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteFees {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub observed_tx: TransactionObserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTx {
    pub memo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionObserved {
    pub tx: TransactionTx,
    pub status: Option<String>,
    pub out_hashes: Option<Vec<String>>,
}

impl TransactionObserved {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status.as_deref() {
            Some("done") => SwapStatus::Completed,
            _ => SwapStatus::Failed, // TODO: Handle refunded status detection later
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData {
    pub router_address: Option<String>,
    pub inbound_address: String,
}

impl RouteData {
    pub fn get_inbound_address(from_asset: &THORChainAsset, quote_inbound_address: Option<String>) -> Result<String, SwapperError> {
        if from_asset.chain == THORChainName::Thorchain {
            Ok(THORCHAIN_INBOUND_ADDRESS.to_string())
        } else {
            quote_inbound_address.ok_or(SwapperError::InvalidRoute)
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InboundAddress {
    pub chain: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub gas_rate: BigInt,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub outbound_fee: BigInt,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub dust_threshold: BigInt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_inbound_address_thorchain() {
        let from_asset = THORChainAsset::from_asset_id("thorchain").unwrap();
        let result = RouteData::get_inbound_address(&from_asset, None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), THORCHAIN_INBOUND_ADDRESS);
    }

    #[test]
    fn test_get_inbound_address_other_chain() {
        let from_asset = THORChainAsset::from_asset_id("ethereum").unwrap();
        let quote_address = Some("0x1234567890abcdef".to_string());
        let result = RouteData::get_inbound_address(&from_asset, quote_address.clone());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), quote_address.unwrap());
    }
}
