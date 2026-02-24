use num_bigint::BigInt;
use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

use super::{
    asset::THORChainAsset,
    chain::THORChainName,
    constants::{THORCHAIN_INBOUND_ADDRESS, ZERO_HASH},
};
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
pub struct TransactionStatus {
    pub tx: TransactionStatusTx,
    pub stages: TransactionStages,
    pub out_txs: Option<Vec<TransactionStatusOutTx>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusTx {
    pub memo: String,
    pub coins: Vec<TransactionCoin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCoin {
    pub asset: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStages {
    pub swap_status: TransactionSwapStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSwapStage {
    pub pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusOutTx {
    pub id: String,
    pub chain: String,
    pub coins: Vec<TransactionCoin>,
}

impl TransactionStatus {
    pub fn swap_status(&self) -> SwapStatus {
        let has_output = self.out_txs.as_ref().is_some_and(|txs| txs.iter().any(|tx| tx.id != ZERO_HASH && !tx.id.is_empty()));

        if !self.stages.swap_status.pending && has_output {
            SwapStatus::Completed
        } else {
            SwapStatus::Pending
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
    pub dust_threshold: BigInt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    const MIN_AMOUNT_PREFIX: &str = "recommended_min_amount_in: ";
    const DUST_THRESHOLD_MSG: &str = "amount less than dust threshold";
    const MIN_SWAP_AMOUNT_MSG: &str = "amount less than min swap amount";

    pub fn is_input_amount_error(&self) -> bool {
        self.message.contains(Self::DUST_THRESHOLD_MSG) || self.message.contains(Self::MIN_SWAP_AMOUNT_MSG)
    }

    pub fn parse_min_amount(&self) -> Option<String> {
        self.message
            .find(Self::MIN_AMOUNT_PREFIX)
            .map(|start| self.message[start + Self::MIN_AMOUNT_PREFIX.len()..].chars().take_while(|c| c.is_ascii_digit()).collect())
            .filter(|s: &String| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_status_completed_ltc_to_tron() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_ltc_to_tron.json")).unwrap();

        assert_eq!(status.swap_status(), SwapStatus::Completed);
        assert_eq!(status.tx.memo, "=:TRON.USDT:TMazs4f2ybMjGf7WXAx4uiRf2T7XtMC6qt:0/1/0:g1:50");
        assert_eq!(status.tx.coins[0].amount, "160661010");

        let out_txs = status.out_txs.unwrap();
        let destination = out_txs.iter().find(|tx| tx.id != ZERO_HASH && !tx.id.is_empty()).unwrap();
        assert_eq!(destination.chain, "TRON");
        assert_eq!(destination.id, "544827704F9AD53F2D33209F73F7CC39C3AA5068481D87316ED189B322784222");
        assert_eq!(destination.coins[0].amount, "7915842900");
    }

    #[test]
    fn test_tx_status_pending_btc_to_tron() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_btc_to_tron_pending.json")).unwrap();

        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert_eq!(status.tx.coins[0].amount, "23516479");
        assert!(status.out_txs.is_none());
    }

    #[test]
    fn test_tx_status_completed_tcy_to_eth_usdt() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_tcy_to_eth_usdt.json")).unwrap();

        assert_eq!(status.swap_status(), SwapStatus::Completed);
        assert_eq!(status.tx.coins[0].amount, "11921829956942");

        let out_txs = status.out_txs.unwrap();
        let destination = out_txs.iter().find(|tx| tx.id != ZERO_HASH && !tx.id.is_empty()).unwrap();
        assert_eq!(destination.chain, "ETH");
        assert_eq!(destination.id, "1D8300FDC5A47ACA3E7D59791180229AE314C86ABA32C14E4975464491865576");
        assert_eq!(destination.coins[0].amount, "380962656200");
    }

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
        let quote_address = "0x1234567890abcdef".to_string();
        let result = RouteData::get_inbound_address(&from_asset, Some(quote_address.clone()));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), quote_address);
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse {
            message: "amount less than min swap amount (recommended_min_amount_in: 50570): invalid request".into(),
        };
        assert!(error.is_input_amount_error());
        assert_eq!(error.parse_min_amount(), Some("50570".into()));

        let error = ErrorResponse {
            message: "amount less than dust threshold: invalid request".into(),
        };
        assert!(error.is_input_amount_error());
        assert_eq!(error.parse_min_amount(), None);
    }
}
