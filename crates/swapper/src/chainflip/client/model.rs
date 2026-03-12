use num_bigint::BigUint;
use primitives::swap::SwapStatus;
use primitives::{AssetId, Chain, TransactionSwapMetadata};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

use crate::{SwapResult, SwapperProvider};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub src_chain: String,
    pub src_asset: String,
    pub dest_chain: String,
    pub dest_asset: String,
    pub is_vault_swap: bool,
    pub dca_enabled: bool,
    pub broker_commission_bps: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncludedFee {
    #[serde(rename = "type")]
    pub fee_type: String,
    pub chain: String,
    pub asset: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcaParams {
    pub number_of_chunks: u32,
    pub chunk_interval_blocks: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub egress_amount: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    pub estimated_duration_seconds: f64,
    #[serde(rename = "type")]
    pub quote_type: String,
    pub deposit_amount: String,
    pub is_vault_swap: bool,
    pub boost_quote: Option<BoostQuote>,
    pub estimated_price: String,
    pub dca_params: Option<DcaParams>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostQuote {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub egress_amount: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    pub estimated_duration_seconds: f64,
    pub estimated_boost_fee_bps: u32,
    pub max_boost_fee_bps: u32,
    pub estimated_price: String,
    pub dca_params: Option<DcaParams>,
}

impl QuoteResponse {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}

impl BoostQuote {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapTxResponse {
    pub state: String,
    pub src_asset: String,
    pub src_chain: String,
    pub dest_asset: String,
    pub dest_chain: String,
    pub deposit: Option<SwapDeposit>,
    pub swap: Option<SwapDetail>,
    pub refund_egress: Option<serde_json::Value>,
}

impl SwapTxResponse {
    pub fn swap_status(&self) -> SwapStatus {
        match self.state.as_str() {
            "COMPLETED" if self.refund_egress.is_some() => SwapStatus::Failed,
            "COMPLETED" => SwapStatus::Completed,
            "FAILED" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapDeposit {
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapDetail {
    pub swapped_output_amount: String,
}

fn chainflip_chain_to_chain(chain: &str) -> Option<Chain> {
    match chain {
        "Ethereum" => Some(Chain::Ethereum),
        "Bitcoin" => Some(Chain::Bitcoin),
        "Solana" => Some(Chain::Solana),
        "Arbitrum" => Some(Chain::Arbitrum),
        _ => None,
    }
}

fn chainflip_asset_to_asset_id(chain: Chain, asset: &str) -> Option<AssetId> {
    use crate::asset::*;
    match (chain, asset) {
        (Chain::Ethereum, "ETH") => Some(AssetId::from_chain(Chain::Ethereum)),
        (Chain::Ethereum, "USDC") => Some(ETHEREUM_USDC.id.clone()),
        (Chain::Ethereum, "USDT") => Some(ETHEREUM_USDT.id.clone()),
        (Chain::Ethereum, "WBTC") => Some(ETHEREUM_WBTC.id.clone()),
        (Chain::Ethereum, "FLIP") => Some(ETHEREUM_FLIP.id.clone()),
        (Chain::Bitcoin, "BTC") => Some(AssetId::from_chain(Chain::Bitcoin)),
        (Chain::Solana, "SOL") => Some(AssetId::from_chain(Chain::Solana)),
        (Chain::Solana, "USDC") => Some(SOLANA_USDC.id.clone()),
        (Chain::Solana, "USDT") => Some(SOLANA_USDT.id.clone()),
        (Chain::Arbitrum, "USDC") => Some(ARBITRUM_USDC.id.clone()),
        (Chain::Arbitrum, "USDT") => Some(ARBITRUM_USDT.id.clone()),
        _ => None,
    }
}

pub fn map_swap_result(response: &SwapTxResponse) -> SwapResult {
    let status = response.swap_status();

    let metadata = if status != SwapStatus::Pending {
        let from_chain = chainflip_chain_to_chain(&response.src_chain);
        let to_chain = chainflip_chain_to_chain(&response.dest_chain);

        from_chain.zip(to_chain).and_then(|(fc, tc)| {
            let from_asset = chainflip_asset_to_asset_id(fc, &response.src_asset)?;
            let to_asset = chainflip_asset_to_asset_id(tc, &response.dest_asset)?;
            let from_value = response.deposit.as_ref()?.amount.clone();
            let to_value = response.swap.as_ref().map(|s| s.swapped_output_amount.clone()).unwrap_or_default();
            Some(TransactionSwapMetadata {
                from_asset,
                from_value,
                to_asset,
                to_value,
                provider: Some(SwapperProvider::Chainflip.as_ref().to_string()),
            })
        })
    } else {
        None
    };

    SwapResult { status, metadata }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use primitives::AssetId;

    fn swap_response(json: &str) -> SwapTxResponse {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    pub fn get_quote_response() {
        let quote_response = serde_json::from_str::<Vec<QuoteResponse>>(include_str!("./test/btc_eth_quote.json")).unwrap();

        assert!(quote_response[0].boost_quote.is_some());
    }

    #[test]
    fn test_map_swap_result_eth_to_btc() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_eth_to_btc.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Ethereum),
                    from_value: "140000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Bitcoin),
                    to_value: "405772".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_usdc_to_sol() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_sol.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                    from_value: "100000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Solana),
                    to_value: "1143469990".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_sol_to_btc() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_sol_to_btc.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Solana),
                    from_value: "150000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Bitcoin),
                    to_value: "17567".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_pending() {
        let result = map_swap_result(&swap_response(include_str!("./test/swap_usdc_to_btc_pending.json")));
        assert_eq!(result.status, SwapStatus::Pending);
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_map_swap_result_refunded() {
        assert_eq!(
            map_swap_result(&swap_response(include_str!("./test/swap_btc_to_usdt_refunded.json"))),
            SwapResult {
                status: SwapStatus::Failed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Bitcoin),
                    from_value: "1508475".to_string(),
                    to_asset: AssetId::from_token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                    to_value: "0".to_string(),
                    provider: Some("chainflip".to_string()),
                }),
            }
        );
    }
}
