mod prefetch;
mod transaction;

use self::{prefetch::PrefetchedSuiData, transaction::build_mctp_transaction};
use crate::mayan::{
    client::MayanClient,
    constants::SDK_VERSION,
    model::{GetSwapSuiParams, MayanMctpQuote, SuiClientSwap},
    tx_builder::route::quote_destination_address,
    wormhole_chain::{self, WormholeChain},
};
use crate::{Quote, RpcProvider, SwapperError, SwapperQuoteData, client_factory::create_sui_client, fees::default_referral_address};
use futures::try_join;
use gem_sui::tx_builder::prepare_transaction_json_replay;
use gem_sui::{ESTIMATION_GAS_BUDGET, gas_budget::GAS_BUDGET_MULTIPLIER};
use std::{fmt::Debug, fmt::Display, sync::Arc};

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanMctpQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    let sender = quote.request.wallet_address.as_str();
    let sui_client = create_sui_client(rpc_provider)?;
    let destination_address = quote_destination_address(quote);
    let mctp_input_contract = route.mctp_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let referrer_address = wormhole_chain::chain_for_name(&route.to_chain)
        .ok()
        .map(default_referral_address)
        .filter(|address| !address.is_empty());
    let prefetched = PrefetchedSuiData::prefetch(&sui_client, sender, route, ESTIMATION_GAS_BUDGET);
    let swap = async {
        let swap = get_swap_transaction(client, quote, route, mctp_input_contract, referrer_address).await?;
        let swap_replay = match &swap {
            Some(swap) => Some(prepare_transaction_json_replay(&sui_client, &swap.tx).await.map_err(sui_error)?),
            None => None,
        };
        Ok::<_, SwapperError>((swap, swap_replay))
    };
    let (prefetched, (swap, swap_replay)) = try_join!(prefetched, swap)?;

    let estimate = build_mctp_transaction(quote, route, &prefetched, destination_address, swap.as_ref(), swap_replay.as_ref(), ESTIMATION_GAS_BUDGET)?;
    let dry_run = sui_client.dry_run(estimate.base64_encoded()).await.map_err(SwapperError::transaction_error)?;
    if dry_run.effects.status.status != "success" {
        let detail = dry_run.effects.status.error.as_deref().unwrap_or("no details available");
        return Err(SwapperError::TransactionError(format!("Sui swap simulation failed: {detail}")));
    }

    let fee = dry_run.effects.gas_used.calculate_gas_budget().map_err(SwapperError::transaction_error)?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;
    let output = build_mctp_transaction(quote, route, &prefetched, destination_address, swap.as_ref(), swap_replay.as_ref(), gas_budget)?;

    Ok(SwapperQuoteData::new_contract(
        String::new(),
        "0".to_string(),
        output.base64_encoded(),
        None,
        Some(gas_budget.to_string()),
    ))
}

async fn get_swap_transaction<C>(
    client: &MayanClient<C>,
    quote: &Quote,
    route: &MayanMctpQuote,
    mctp_input_contract: &str,
    referrer_address: Option<String>,
) -> Result<Option<SuiClientSwap>, SwapperError>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    if route.from_token.contract.as_str() == mctp_input_contract {
        return Ok(None);
    }

    client
        .post_swap(
            "/get-swap/sui",
            GetSwapSuiParams {
                amount_in64: route.effective_amount_in64.clone(),
                input_coin_type: route.from_token.contract.clone(),
                middle_coin_type: mctp_input_contract.to_string(),
                user_wallet: quote.request.wallet_address.clone(),
                with_wh_fee: route.has_auction == Some(true) || route.cheaper_chain.as_deref() != Some(WormholeChain::Sui.name()),
                referrer_address,
                slippage_bps: route.slippage_bps,
                chain_name: route.from_chain.clone(),
                sdk_version: SDK_VERSION,
            },
        )
        .await
        .map(Some)
}

fn sui_error(err: impl Display) -> SwapperError {
    SwapperError::TransactionError(format!("Sui transaction error: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_sui::SUI_COIN_TYPE;

    #[test]
    fn test_mctp_route_amounts() {
        let route = MayanMctpQuote::mock();
        assert_eq!(transaction::bridge_amount(&route, route.mctp_input_contract.as_deref().unwrap()).unwrap(), 1000000);

        let mut route = route;
        route.common.from_token.contract = SUI_COIN_TYPE.to_string();
        route.min_middle_amount = Some(serde_json::json!(0.99));
        assert_eq!(transaction::bridge_amount(&route, route.mctp_input_contract.as_deref().unwrap()).unwrap(), 990000);
    }
}
