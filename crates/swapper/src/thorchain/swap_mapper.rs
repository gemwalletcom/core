use primitives::TransactionSwapMetadata;

use super::chain::THORChainName;
use super::constants::ZERO_HASH;
use super::model::TransactionStatus;
use crate::{SwapResult, SwapperProvider};

pub fn map_swap_result(response: &TransactionStatus) -> SwapResult {
    let status = response.swap_status();

    let Some(ref tx) = response.tx else {
        return SwapResult { status, metadata: None };
    };

    let Some(chain) = THORChainName::from_symbol(&tx.chain).map(|n| n.chain()) else {
        return SwapResult { status, metadata: None };
    };

    let from_coin = tx.coins.first();
    let from_asset = from_coin.and_then(|c| c.resolve_asset_id());
    let from_value = from_coin.and_then(|c| c.native_value(chain));

    let out_coin = response
        .out_txs
        .as_ref()
        .and_then(|out_txs| out_txs.iter().find(|t| t.id != ZERO_HASH && !t.id.is_empty()).and_then(|t| t.coins.first()));
    let to_asset = out_coin.and_then(|c| c.resolve_asset_id());
    let to_value = out_coin.and_then(|c| to_asset.as_ref().and_then(|a| c.native_value(a.chain)));

    let metadata = match (from_asset, from_value, to_asset, to_value) {
        (Some(from_asset), Some(from_value), Some(to_asset), Some(to_value)) => Some(TransactionSwapMetadata {
            from_asset,
            from_value,
            to_asset,
            to_value,
            provider: Some(SwapperProvider::Thorchain.as_ref().to_string()),
        }),
        _ => None,
    };

    SwapResult { status, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::ETHEREUM_USDT_TOKEN_ID;
    use primitives::{AssetId, Chain, swap::SwapStatus};

    fn status(json: &str) -> TransactionStatus {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result_ltc_to_tron_usdt() {
        let response = status(include_str!("testdata/tx_status_ltc_to_tron_usdt.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Litecoin.as_asset_id(),
                    from_value: "160661010".to_string(),
                    to_asset: AssetId::from_token(Chain::Tron, crate::asset::TRON_USDT_TOKEN_ID),
                    to_value: "79158429".to_string(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_ltc_to_eth() {
        let response = status(include_str!("testdata/tx_status_ltc_to_eth.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Litecoin.as_asset_id(),
                    from_value: "5000000".to_string(),
                    to_asset: Chain::Ethereum.as_asset_id(),
                    to_value: "1243680000000000".to_string(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_btc_to_tron_pending() {
        let response = status(include_str!("testdata/tx_status_btc_to_tron_pending.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: None
            }
        );
    }

    #[test]
    fn test_map_swap_result_bnb_to_eth_usdt() {
        let response = status(include_str!("testdata/tx_status_bnb_to_eth_usdt.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::SmartChain.as_asset_id(),
                    from_value: "21300000000000000".to_string(),
                    to_asset: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
                    to_value: "12973781".to_string(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_tcy_to_eth_usdt() {
        let response = status(include_str!("testdata/tx_status_tcy_to_eth_usdt.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_token(Chain::Thorchain, "tcy"),
                    from_value: "11921829956942".to_string(),
                    to_asset: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
                    to_value: "3809626562".to_string(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }
}
