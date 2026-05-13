use super::{
    error::sui_error,
    swap::shared_object_ids,
    transaction::{BuildInput, build_transaction},
};
use crate::{
    Quote, SwapperError, SwapperQuoteData,
    cetus::{constants::PINNED_VERSIONS, model::RouterData},
    fees::ReferralFee,
};
use gem_client::ClientBounds;
use gem_sui::{ESTIMATION_GAS_BUDGET, SuiClient, gas_budget::GAS_BUDGET_MULTIPLIER, tx_builder::PrefetchedTransactionData};

pub async fn build_quote_data<C: ClientBounds>(client: &SuiClient<C>, quote: &Quote, router: &RouterData, referral_fee: &ReferralFee) -> Result<SwapperQuoteData, SwapperError> {
    let sender = quote.request.wallet_address.as_str();
    let from_coin_type = router.paths.first().ok_or(SwapperError::InvalidRoute)?.from.clone();
    let target_coin_type = router.paths.last().ok_or(SwapperError::InvalidRoute)?.target.clone();
    let amount = quote.from_value.parse::<u64>()?;
    let prefetched = PrefetchedTransactionData::prefetch(
        client,
        sender,
        &from_coin_type,
        &target_coin_type,
        shared_object_ids(router)?,
        &PINNED_VERSIONS,
        ESTIMATION_GAS_BUDGET,
    )
    .await
    .map_err(sui_error)?;

    let input = BuildInput {
        transaction: prefetched.transaction.clone(),
        from_coin_type: &from_coin_type,
        target_coin_type: &target_coin_type,
        amount,
        from_coins: &prefetched.input_coins,
        target_merge_coin: prefetched.output_coin.as_ref(),
    };

    let estimate = build_transaction(&prefetched.resolver, quote, router, referral_fee, &input)?;
    let dry_run = client
        .dry_run(estimate.base64_encoded())
        .await
        .map_err(|err| SwapperError::TransactionError(err.to_string()))?;
    if dry_run.effects.status.status != "success" {
        let detail = dry_run.effects.status.error.as_deref().unwrap_or("no details available");
        return Err(SwapperError::TransactionError(format!("Sui swap simulation failed: {detail}")));
    }

    let fee = dry_run
        .effects
        .gas_used
        .calculate_gas_budget()
        .map_err(|err| SwapperError::TransactionError(err.to_string()))?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;
    let output = build_transaction(&prefetched.resolver, quote, router, referral_fee, &input.with_gas_budget(gas_budget))?;

    Ok(SwapperQuoteData::new_contract(
        String::new(),
        "0".to_string(),
        output.base64_encoded(),
        None,
        Some(gas_budget.to_string()),
    ))
}
