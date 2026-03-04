use primitives::TransactionSwapMetadata;

use super::{asset::map_currency_to_asset_id, chain::RelayChain, model::RelayRequest};
use crate::{SwapResult, SwapperProvider};

pub fn map_swap_result(request: &RelayRequest) -> SwapResult {
    let metadata = request.data.as_ref().and_then(|d| d.metadata.as_ref()).and_then(|m| {
        let currency_in = m.currency_in.as_ref()?;
        let currency_out = m.currency_out.as_ref()?;
        let from_chain = RelayChain::from_chain_id(currency_in.currency.chain_id)?.to_chain();
        let to_chain = RelayChain::from_chain_id(currency_out.currency.chain_id)?.to_chain();
        Some(TransactionSwapMetadata {
            from_asset: map_currency_to_asset_id(from_chain, &currency_in.currency.address),
            from_value: currency_in.amount.clone()?,
            to_asset: map_currency_to_asset_id(to_chain, &currency_out.currency.address),
            to_value: currency_out.amount.clone()?,
            provider: Some(SwapperProvider::Relay.as_ref().to_string()),
        })
    });

    SwapResult {
        status: request.status.clone().into_swap_status(),
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::{RelayCurrencyDetail, RelayRequest, RelayRequestMetadata, RelayRequestsResponse, RelayStatus};
    use primitives::{AssetId, Chain, swap::SwapStatus};

    #[test]
    fn test_map_swap_result_evm_to_evm() {
        let request = RelayRequest::mock(
            RelayStatus::Success,
            Some(RelayRequestMetadata {
                currency_in: Some(RelayCurrencyDetail::mock("0x0000000000000000000000000000000000000000", 1, "1000000000000000000")),
                currency_out: Some(RelayCurrencyDetail::mock("0x0000000000000000000000000000000000000000", 8453, "999000000000000000")),
            }),
        );

        let result = map_swap_result(&request);

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(metadata.from_value, "1000000000000000000");
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Base));
        assert_eq!(metadata.to_value, "999000000000000000");
        assert_eq!(metadata.provider, Some("relay".to_string()));
    }

    #[test]
    fn test_map_swap_result_status() {
        let pending = map_swap_result(&RelayRequest::mock(RelayStatus::Pending, None));
        assert_eq!(pending.status, SwapStatus::Pending);
        assert!(pending.metadata.is_none());

        let failed = map_swap_result(&RelayRequest::mock(RelayStatus::Failed, None));
        assert_eq!(failed.status, SwapStatus::Failed);
        assert!(failed.metadata.is_none());
    }

    #[test]
    fn test_map_swap_result_eth_to_btc() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_eth_to_btc.json")).unwrap();
        let request = response.requests.first().unwrap();
        let result = map_swap_result(request);

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(metadata.from_value, "10000000000000000");
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Bitcoin));
        assert_eq!(metadata.to_value, "28619");
        assert_eq!(metadata.provider, Some("relay".to_string()));
    }

    #[test]
    fn test_map_swap_result_bsc_usdt_to_sol() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_bsc_usdt_to_sol.json")).unwrap();
        let request = response.requests.first().unwrap();
        let result = map_swap_result(request);

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::SmartChain, "0x55d398326f99059fF775485246999027B3197955"));
        assert_eq!(metadata.from_value, "6000000000000000000");
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Solana));
        assert_eq!(metadata.to_value, "74432990");
    }
}
