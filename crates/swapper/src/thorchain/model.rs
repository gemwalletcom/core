use num_bigint::BigInt;
use primitives::{Asset, AssetId, Chain, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

use super::{
    asset::{THORChainAsset, value_to},
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
    pub tx: Option<TransactionStatusTx>,
    pub stages: TransactionStages,
    pub out_txs: Option<Vec<TransactionStatusOutTx>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusTx {
    pub chain: String,
    pub memo: String,
    pub coins: Vec<TransactionCoin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCoin {
    pub asset: String,
    pub amount: String,
    pub decimals: Option<i32>,
}

impl TransactionCoin {
    pub fn native_value(&self, chain: Chain) -> Option<String> {
        let decimals = self
            .decimals
            .or_else(|| if self.is_native_asset() { Some(Asset::from_chain(chain).decimals) } else { None })?;
        Some(value_to(&self.amount, decimals).to_string())
    }

    pub fn resolve_asset_id(&self) -> Option<AssetId> {
        let (chain_str, rest) = self.asset.split_once('.')?;
        let chain_name = THORChainName::from_symbol(chain_str)?;
        let chain = chain_name.chain();
        let key = rest.split_once('-').map_or(rest, |(_, addr)| addr);
        match THORChainAsset::from(chain_name, key).and_then(|a| a.token_id) {
            Some(token_id) => Some(AssetId::from_token(chain, &token_id)),
            None if self.is_native_asset() => Some(AssetId::from_chain(chain)),
            None => None,
        }
    }

    fn is_native_asset(&self) -> bool {
        !self.asset.contains('-')
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStages {
    pub swap_status: Option<TransactionSwapStage>,
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
        let has_output = self.out_txs.as_ref().is_some_and(|txs| !txs.is_empty());
        let swap_done = self.stages.swap_status.as_ref().is_some_and(|s| !s.pending);

        if swap_done && has_output { SwapStatus::Completed } else { SwapStatus::Pending }
    }

    pub fn destination_tx(&self) -> Option<&TransactionStatusOutTx> {
        self.out_txs
            .as_ref()
            .and_then(|txs| txs.iter().find(|x| x.id != ZERO_HASH && !x.id.is_empty()).or_else(|| txs.first()))
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
    pub address: String,
    pub router: Option<String>,
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
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_ltc_to_tron_usdt.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);

        let destination = status.destination_tx().unwrap();
        assert_eq!(destination.chain, "TRON");
        assert_eq!(destination.id, "544827704F9AD53F2D33209F73F7CC39C3AA5068481D87316ED189B322784222");
        assert_eq!(destination.coins[0].amount, "7915842900");
    }

    #[test]
    fn test_tx_status_pending_btc_to_tron() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_btc_to_tron_pending.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert!(status.destination_tx().is_none());
    }

    #[test]
    fn test_tx_status_completed_tcy_to_eth_usdt() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_tcy_to_eth_usdt.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);

        let destination = status.destination_tx().unwrap();
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
    fn test_native_value() {
        let native = TransactionCoin {
            asset: "LTC.LTC".to_string(),
            amount: "160661010".to_string(),
            decimals: None,
        };
        assert_eq!(native.native_value(Chain::Litecoin), Some("160661010".to_string()));

        let native_18 = TransactionCoin {
            asset: "ETH.ETH".to_string(),
            amount: "2509674".to_string(),
            decimals: None,
        };
        assert_eq!(native_18.native_value(Chain::Ethereum), Some("25096740000000000".to_string()));

        let token_with_decimals = TransactionCoin {
            asset: "ETH.USDT-0xDAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            amount: "380962656200".to_string(),
            decimals: Some(6),
        };
        assert_eq!(token_with_decimals.native_value(Chain::Ethereum), Some("3809626562".to_string()));

        let token_no_decimals = TransactionCoin {
            asset: "ETH.USDT-0xDAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            amount: "380962656200".to_string(),
            decimals: None,
        };
        assert_eq!(token_no_decimals.native_value(Chain::Ethereum), None);
    }

    #[test]
    fn test_tx_status_completed_eth_usdt_to_rune() {
        let status: TransactionStatus = serde_json::from_str(include_str!("testdata/tx_status_eth_usdt_to_rune.json")).unwrap();
        assert_eq!(status.swap_status(), SwapStatus::Completed);

        let destination = status.destination_tx().unwrap();
        assert_eq!(destination.chain, "THOR");
        assert_eq!(destination.coins[0].amount, "2096315169517");
    }

    #[test]
    fn test_tx_status_not_observed() {
        let json = r#"{"stages":{"inbound_observed":{"started":false,"final_count":0,"completed":false}}}"#;
        let status: TransactionStatus = serde_json::from_str(json).unwrap();

        assert_eq!(status.swap_status(), SwapStatus::Pending);
        assert!(status.tx.is_none());
        assert!(status.out_txs.is_none());
    }

    #[test]
    fn test_resolve_asset_id() {
        fn coin(asset: &str) -> TransactionCoin {
            TransactionCoin {
                asset: asset.to_string(),
                amount: "0".to_string(),
                decimals: None,
            }
        }

        assert_eq!(coin("LTC.LTC").resolve_asset_id(), Some(Chain::Litecoin.as_asset_id()));
        assert_eq!(coin("ETH.ETH").resolve_asset_id(), Some(Chain::Ethereum.as_asset_id()));
        assert_eq!(coin("BTC.BTC").resolve_asset_id(), Some(Chain::Bitcoin.as_asset_id()));
        assert_eq!(coin("THOR.RUNE").resolve_asset_id(), Some(Chain::Thorchain.as_asset_id()));
        assert_eq!(
            coin("ETH.USDT-0XDAC17F958D2EE523A2206206994597C13D831EC7").resolve_asset_id(),
            Some(AssetId::from_token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7"))
        );
        assert_eq!(
            coin("TRON.USDT-TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t").resolve_asset_id(),
            Some(AssetId::from_token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"))
        );
        assert_eq!(coin("THOR.TCY").resolve_asset_id(), Some(AssetId::from_token(Chain::Thorchain, "tcy")));
        assert_eq!(coin("ETH.UNKNOWN-0x1234567890abcdef1234567890abcdef12345678").resolve_asset_id(), None);
        assert_eq!(coin("INVALID").resolve_asset_id(), None);
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
