use primitives::{AssetId, Chain, TransactionSwapMetadata};

use super::chain::THORChainName;
use super::constants::ZERO_HASH;
use super::memo::ThorchainMemo;
use super::model::TransactionStatus;
use crate::{SwapResult, SwapperProvider};

pub fn map_swap_result(response: &TransactionStatus, chain: Chain) -> SwapResult {
    let status = response.swap_status();
    let memo = ThorchainMemo::parse(&response.tx.memo);
    let destination_chain = memo.as_ref().and_then(|m| m.destination_chain());
    let from_value = response.tx.coins.first().and_then(|c| c.native_value(chain)).unwrap_or_default();

    let out_transaction = response.out_txs.as_ref().and_then(|out_txs| {
        let chain_name = destination_chain.and_then(|c| THORChainName::from_chain(&c)).map(|n| n.long_name().to_string());
        out_txs
            .iter()
            .find(|t| t.id != ZERO_HASH && !t.id.is_empty() && chain_name.as_ref().is_none_or(|name| t.chain == *name))
    });

    let to_chain = destination_chain.or_else(|| out_transaction.and_then(|t| THORChainName::from_symbol(&t.chain).map(|n| n.chain())));
    let to_value = out_transaction
        .and_then(|t| t.coins.first().and_then(|c| to_chain.and_then(|tc| c.native_value(tc))))
        .unwrap_or_default();

    let metadata = to_chain.map(|to| TransactionSwapMetadata {
        from_asset: AssetId::from_chain(chain),
        from_value,
        to_asset: AssetId::from_chain(to),
        to_value,
        provider: Some(SwapperProvider::Thorchain.as_ref().to_string()),
    });

    SwapResult { status, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::swap::SwapStatus;

    fn status(json: &str) -> TransactionStatus {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result_ltc_to_tron() {
        let response = status(include_str!("testdata/tx_status_ltc_to_tron.json"));

        assert_eq!(
            map_swap_result(&response, Chain::Litecoin),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Litecoin.as_asset_id(),
                    from_value: "160661010".to_string(),
                    to_asset: Chain::Tron.as_asset_id(),
                    to_value: "79158429".to_string(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_btc_to_tron_pending() {
        let response = status(include_str!("testdata/tx_status_btc_to_tron_pending.json"));

        assert_eq!(
            map_swap_result(&response, Chain::Bitcoin),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Bitcoin.as_asset_id(),
                    from_value: "23516479".to_string(),
                    to_asset: Chain::Tron.as_asset_id(),
                    to_value: String::new(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_tcy_to_eth_usdt() {
        let response = status(include_str!("testdata/tx_status_tcy_to_eth_usdt.json"));

        assert_eq!(
            map_swap_result(&response, Chain::Thorchain),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Thorchain.as_asset_id(),
                    from_value: "11921829956942".to_string(),
                    to_asset: Chain::Ethereum.as_asset_id(),
                    to_value: "3809626562".to_string(),
                    provider: Some("thorchain".to_string()),
                }),
            }
        );
    }
}
