use alloy_primitives::U256;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use gem_solana::{constants::STATIC_BASE_FEE, provider::preload_mapper::calculate_fee_rates, rpc::client::SolanaClient};
use num_traits::ToPrimitive;
use primitives::{Asset, Chain, FeePriority, TransactionInputType};
use std::{fmt::Debug, str::FromStr};

use crate::{QuoteRequest, SwapperError};
use primitives::swap::SwapData;

const SOLANA_SWAP_FEE_MULTIPLIER: u64 = 2;

fn resolve_with_fee(request: &QuoteRequest, reserved_fee_lamports: u64) -> Result<(String, QuoteRequest), SwapperError> {
    if !request.should_use_max_native_amount() {
        return Ok((request.value.clone(), request.clone()));
    }

    let value = U256::from_str(&request.value)?;
    let reserved_fee = U256::from(reserved_fee_lamports);

    if value <= reserved_fee {
        return Err(SwapperError::InputAmountTooSmall);
    }

    let resolved_value = value - reserved_fee;
    let mut adjusted_request = request.clone();
    adjusted_request.value = resolved_value.to_string();

    Ok((adjusted_request.value.clone(), adjusted_request))
}

async fn solana_fee_for_kind<C>(rpc_client: &JsonRpcClient<C>, is_swap: bool) -> u64
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let solana_client = SolanaClient::new(rpc_client.clone());
    let prioritization_fees = solana_client.get_recent_prioritization_fees().await.unwrap_or_default();
    let input_type = if is_swap {
        TransactionInputType::Swap(Asset::from_chain(Chain::Solana), Asset::from_chain(Chain::Solana), SwapData::mock())
    } else {
        TransactionInputType::Transfer(Asset::from_chain(Chain::Solana))
    };

    let fee_rates = calculate_fee_rates(&input_type, &prioritization_fees);
    let normal_fee = fee_rates.iter().find(|rate| rate.priority == FeePriority::Normal).or_else(|| fee_rates.first());

    let multiplier = if is_swap { SOLANA_SWAP_FEE_MULTIPLIER } else { 1 };

    normal_fee
        .and_then(|rate| rate.gas_price_type.total_fee().to_u64().map(|base| base.saturating_mul(multiplier)))
        .unwrap_or(STATIC_BASE_FEE)
}

pub async fn get_max_swap_amount<C>(request: &QuoteRequest, rpc_client: &JsonRpcClient<C>) -> Result<(String, QuoteRequest), SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    if !request.should_use_max_native_amount() {
        return Ok((request.value.clone(), request.clone()));
    }

    let reserved_fee = solana_fee_for_kind(rpc_client, true).await;
    resolve_with_fee(request, reserved_fee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperMode, SwapperQuoteAsset, models::Options};
    use primitives::Chain;

    fn build_request(value: &str, use_max_amount: bool) -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset {
                id: Chain::Solana.as_asset_id().to_string(),
                symbol: "SOL".into(),
                decimals: 9,
            },
            to_asset: SwapperQuoteAsset {
                id: Chain::Ethereum.as_asset_id().to_string(),
                symbol: "ETH".into(),
                decimals: 18,
            },
            wallet_address: "wallet".into(),
            destination_address: "destination".into(),
            value: value.to_string(),
            mode: SwapperMode::ExactIn,
            options: Options {
                use_max_amount,
                ..Default::default()
            },
        }
    }

    #[test]
    fn reserves_fee_when_use_max_enabled() {
        let request = build_request("100000", true);
        let (resolved_value, adjusted_request) = resolve_with_fee(&request, 5_000).expect("expected amount to resolve");

        assert_eq!(resolved_value, "95000");
        assert_eq!(adjusted_request.value, resolved_value);
    }

    #[test]
    fn keeps_value_when_use_max_disabled() {
        let request = build_request("100000", false);
        let (resolved_value, adjusted_request) = resolve_with_fee(&request, 5_000).expect("expected amount to resolve");

        assert_eq!(resolved_value, "100000");
        assert_eq!(adjusted_request.value, "100000");
    }

    #[test]
    fn errors_when_value_is_under_fee() {
        let request = build_request("5000", true);
        match resolve_with_fee(&request, 5_000) {
            Err(SwapperError::InputAmountTooSmall) => {}
            other => panic!("expected InputAmountTooSmall, got {:?}", other),
        }
    }
}
