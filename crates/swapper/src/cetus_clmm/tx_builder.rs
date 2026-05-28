use super::{
    constants::{
        CETUS_CLMM_PUBLISHED_AT, CETUS_GLOBAL_CONFIG, CETUS_PARTNER, CETUS_PARTNER_INIT_VERSION, CETUS_POOLS_REGISTRY, CETUS_SHARED_INIT_VERSION, FUNCTION_CALCULATE_SWAP_RESULT,
        FUNCTION_CALCULATED_SWAP_RESULT_AMOUNT_OUT, FUNCTION_FLASH_SWAP_WITH_PARTNER, FUNCTION_NEW_POOL_KEY, FUNCTION_POOL_ID, FUNCTION_POOL_SIMPLE_INFO,
        FUNCTION_REPAY_FLASH_SWAP_WITH_PARTNER, MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64, MODULE_FACTORY, MODULE_POOL,
    },
    model::{FeeSide, Hop, PoolRoute},
};
use crate::{Quote, SwapperError, SwapperQuoteData, fees::ReferralFee, fees::apply_slippage_in_bp};
use gem_sui::{
    EMPTY_ADDRESS, ESTIMATION_GAS_BUDGET, SuiClient,
    address::SuiAddress,
    gas_budget::GAS_BUDGET_MULTIPLIER,
    is_sui_coin,
    models::{Coin, OwnedCoins, TxOutput},
    sui_clock_object_input,
    tx_builder::{
        ObjectResolver, PrefetchedTransactionData, TransactionBuilderInput, balance_value, balance_zero, build_input_coin, destroy_zero_balance, finish_transaction, from_balance,
        into_balance, move_call,
    },
};
use primitives::Address as AddressTrait;
use std::{collections::HashMap, fmt::Display};
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};
use sui_types::{Address, Digest};

#[derive(Clone)]
pub(super) struct BuildInput<'a> {
    pub transaction: TransactionBuilderInput,
    pub amount: u64,
    pub from_coins: &'a OwnedCoins<Coin>,
}

impl BuildInput<'_> {
    fn with_gas_budget(&self, gas_budget: u64) -> Self {
        Self {
            transaction: self.transaction.with_gas_budget(gas_budget),
            ..self.clone()
        }
    }
}

pub(super) async fn build_quote_data(
    client: &SuiClient,
    quote: &Quote,
    route: &PoolRoute,
    referral_fee: &ReferralFee,
    published_at: &str,
) -> Result<SwapperQuoteData, SwapperError> {
    let sender = quote.request.wallet_address.as_str();
    let amount = quote.from_value.parse::<u64>()?;
    let mut pinned = HashMap::from([
        (CETUS_GLOBAL_CONFIG.to_string(), CETUS_SHARED_INIT_VERSION),
        (CETUS_PARTNER.to_string(), CETUS_PARTNER_INIT_VERSION),
    ]);
    let mut object_ids = vec![CETUS_GLOBAL_CONFIG.to_string(), CETUS_PARTNER.to_string()];
    for hop in &route.hops {
        pinned.insert(hop.pool_id.clone(), hop.pool_init_version);
        object_ids.push(hop.pool_id.clone());
    }
    let PrefetchedTransactionData {
        transaction,
        input_coins,
        resolver,
        ..
    } = PrefetchedTransactionData::prefetch(client, sender, route.input_coin_type(), None, object_ids, &pinned, ESTIMATION_GAS_BUDGET)
        .await
        .map_err(SwapperError::transaction_error)?;

    let input = BuildInput {
        transaction,
        amount,
        from_coins: &input_coins,
    };

    let estimate = build_transaction(&resolver, quote, route, referral_fee, published_at, &input)?;
    let dry_run = client.dry_run(estimate.base64_encoded()).await.map_err(SwapperError::transaction_error)?;
    if dry_run.effects.status.status != "success" {
        let detail = dry_run.effects.status.error.as_deref().unwrap_or("no details available");
        if detail.contains("checked_package_version") {
            return Err(SwapperError::TransactionError(
                "Cetus CLMM was upgraded since this app was built; on-chain Cetus quotes are temporarily unavailable.".into(),
            ));
        }
        return Err(SwapperError::TransactionError(format!("Sui Cetus CLMM swap simulation failed: {detail}")));
    }

    let fee = dry_run.effects.gas_used.calculate_gas_budget().map_err(SwapperError::transaction_error)?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;
    let output = build_transaction(&resolver, quote, route, referral_fee, published_at, &input.with_gas_budget(gas_budget))?;

    Ok(SwapperQuoteData::new_contract(
        String::new(),
        "0".to_string(),
        output.base64_encoded(),
        None,
        Some(gas_budget.to_string()),
    ))
}

pub(super) fn build_batch_quote_inspect(quotes: &[(&Hop, u64)]) -> Result<Vec<u8>, SwapperError> {
    let mut txb = TransactionBuilder::new();
    for (hop, amount_in) in quotes {
        let pool = txb.object(shared_object_input(&hop.pool_id, hop.pool_init_version, false)?);
        let a2b = txb.pure(&hop.a2b);
        let by_amount_in = txb.pure(&true);
        let amount = txb.pure(amount_in);
        move_call(
            &mut txb,
            cetus_clmm_publish_at(),
            MODULE_POOL,
            FUNCTION_CALCULATE_SWAP_RESULT,
            &[&hop.coin_a, &hop.coin_b],
            vec![pool, a2b, by_amount_in, amount],
        )
        .map_err(error)?;
    }
    inspect_transaction_kind_bytes(txb)
}

pub(super) fn build_batch_multi_hop_quote_inspect(routes: &[(&Hop, &Hop, u64)]) -> Result<Vec<u8>, SwapperError> {
    let mut txb = TransactionBuilder::new();
    for (hop1, hop2, amount_in) in routes {
        let pool1 = txb.object(shared_object_input(&hop1.pool_id, hop1.pool_init_version, false)?);
        let a2b1 = txb.pure(&hop1.a2b);
        let by_amount_in_1 = txb.pure(&true);
        let amount = txb.pure(amount_in);
        let csr1 = move_call(
            &mut txb,
            cetus_clmm_publish_at(),
            MODULE_POOL,
            FUNCTION_CALCULATE_SWAP_RESULT,
            &[&hop1.coin_a, &hop1.coin_b],
            vec![pool1, a2b1, by_amount_in_1, amount],
        )
        .map_err(error)?;
        let amount2 = move_call(&mut txb, cetus_clmm_publish_at(), MODULE_POOL, FUNCTION_CALCULATED_SWAP_RESULT_AMOUNT_OUT, &[], vec![csr1]).map_err(error)?;
        let pool2 = txb.object(shared_object_input(&hop2.pool_id, hop2.pool_init_version, false)?);
        let a2b2 = txb.pure(&hop2.a2b);
        let by_amount_in_2 = txb.pure(&true);
        move_call(
            &mut txb,
            cetus_clmm_publish_at(),
            MODULE_POOL,
            FUNCTION_CALCULATE_SWAP_RESULT,
            &[&hop2.coin_a, &hop2.coin_b],
            vec![pool2, a2b2, by_amount_in_2, amount2],
        )
        .map_err(error)?;
    }
    inspect_transaction_kind_bytes(txb)
}

pub(super) fn build_pool_id_inspect(coin_a: &str, coin_b: &str, tick_spacing: u32) -> Result<Vec<u8>, SwapperError> {
    let mut txb = TransactionBuilder::new();
    let tick = txb.pure(&tick_spacing);
    let key = move_call(&mut txb, cetus_clmm_publish_at(), MODULE_FACTORY, FUNCTION_NEW_POOL_KEY, &[coin_a, coin_b], vec![tick]).map_err(error)?;
    let pools = txb.object(shared_object_input(CETUS_POOLS_REGISTRY, CETUS_SHARED_INIT_VERSION, false)?);
    let info = move_call(&mut txb, cetus_clmm_publish_at(), MODULE_FACTORY, FUNCTION_POOL_SIMPLE_INFO, &[], vec![pools, key]).map_err(error)?;
    move_call(&mut txb, cetus_clmm_publish_at(), MODULE_FACTORY, FUNCTION_POOL_ID, &[], vec![info]).map_err(error)?;
    inspect_transaction_kind_bytes(txb)
}

struct PendingRepay<'a> {
    hop: &'a Hop,
    pool: Argument,
    pay_input: Argument,
    pay_zero_output: Argument,
    receipt: Argument,
}

fn build_transaction(
    resolver: &ObjectResolver,
    quote: &Quote,
    route: &PoolRoute,
    referral_fee: &ReferralFee,
    published_at: &str,
    input: &BuildInput<'_>,
) -> Result<TxOutput, SwapperError> {
    let mut txb = TransactionBuilder::new();
    let published_at = SuiAddress::from_str(published_at).map(Address::from)?;
    let input_coin = build_input_coin(&mut txb, route.input_coin_type(), input.amount, input.from_coins).map_err(error)?;
    let (swap_coin, swap_amount) = match route.fee_side {
        FeeSide::Input => {
            let after_fee = pay_referral_fee(&mut txb, input_coin, route.fee_amount, referral_fee)?;
            let swap_amount = input
                .amount
                .checked_sub(route.fee_amount)
                .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus CLMM referral fee exceeds input amount".into()))?;
            (after_fee, swap_amount)
        }
        FeeSide::Output => (input_coin, input.amount),
    };

    let first_hop = route.hops.first().ok_or_else(|| SwapperError::TransactionError("Cetus CLMM route has no hops".into()))?;
    if first_hop.amount_in != swap_amount {
        return Err(SwapperError::TransactionError("Cetus CLMM first hop amount mismatch".into()));
    }

    let global_config = resolver.shared_object(&mut txb, CETUS_GLOBAL_CONFIG, false).map_err(error)?;
    let partner = resolver.shared_object(&mut txb, CETUS_PARTNER, true).map_err(error)?;
    let clock = txb.object(sui_clock_object_input());

    let mut current_balance = into_balance(&mut txb, route.input_coin_type(), swap_coin).map_err(error)?;
    let mut current_coin_type = route.input_coin_type().to_string();
    let mut pending_repays: Vec<PendingRepay<'_>> = Vec::new();

    for (idx, hop) in route.hops.iter().enumerate() {
        let pool = resolver.shared_object(&mut txb, &hop.pool_id, true).map_err(error)?;
        let pay_input = current_balance;
        let amount_arg = if idx == 0 {
            txb.pure(&hop.amount_in)
        } else {
            balance_value(&mut txb, &current_coin_type, current_balance).map_err(error)?
        };

        let zero_output = balance_zero(&mut txb, hop.output_coin_type()).map_err(error)?;
        let a2b_arg = txb.pure(&hop.a2b);
        let by_amount_in_arg = txb.pure(&true);
        let sqrt_price_limit_arg = txb.pure(&sqrt_price_limit_with_slippage(hop, quote.request.options.slippage.bps));

        let returns = move_call(
            &mut txb,
            published_at,
            MODULE_POOL,
            FUNCTION_FLASH_SWAP_WITH_PARTNER,
            &[&hop.coin_a, &hop.coin_b],
            vec![global_config, pool, partner, a2b_arg, by_amount_in_arg, amount_arg, sqrt_price_limit_arg, clock],
        )
        .map_err(error)?
        .to_nested(3);
        let (recv_a, recv_b, receipt) = match returns.as_slice() {
            [recv_a, recv_b, receipt] => (*recv_a, *recv_b, *receipt),
            _ => return Err(SwapperError::TransactionError("Cetus CLMM flash_swap did not return expected outputs".into())),
        };

        let (output_balance, empty_balance) = hop.order_by_direction(recv_b, recv_a);
        destroy_zero_balance(&mut txb, hop.input_coin_type(), empty_balance).map_err(error)?;

        pending_repays.push(PendingRepay {
            hop,
            pool,
            pay_input,
            pay_zero_output: zero_output,
            receipt,
        });

        current_balance = output_balance;
        current_coin_type = hop.output_coin_type().to_string();
    }

    for repay in pending_repays {
        let (pay_a, pay_b) = repay.hop.order_by_direction(repay.pay_input, repay.pay_zero_output);
        move_call(
            &mut txb,
            published_at,
            MODULE_POOL,
            FUNCTION_REPAY_FLASH_SWAP_WITH_PARTNER,
            &[&repay.hop.coin_a, &repay.hop.coin_b],
            vec![global_config, repay.pool, partner, pay_a, pay_b, repay.receipt],
        )
        .map_err(error)?;
    }

    let sender = SuiAddress::from_str(&quote.request.wallet_address).map(Address::from)?;
    let output_coin = from_balance(&mut txb, route.output_coin_type(), current_balance).map_err(error)?;

    let pay_output_fee = match route.fee_side {
        FeeSide::Output => route.fee_amount > 0,
        FeeSide::Input => false,
    };
    if pay_output_fee {
        let fee_amount_arg = txb.pure(&route.fee_amount);
        let fee_coin = txb
            .split_coins(output_coin, vec![fee_amount_arg])
            .pop()
            .ok_or_else(|| SwapperError::TransactionError("Cetus CLMM output fee split failed".into()))?;
        let recipient = SuiAddress::from_str(&referral_fee.address).map(Address::from)?;
        transfer_coin(&mut txb, fee_coin, recipient);
    }

    let min_out = apply_slippage_in_bp(&route.net_amount_out(), quote.request.options.slippage.bps);
    let min_out_arg = txb.pure(&min_out);
    let split_off = txb
        .split_coins(output_coin, vec![min_out_arg])
        .pop()
        .ok_or_else(|| SwapperError::TransactionError("Cetus CLMM min-out split failed".into()))?;
    txb.merge_coins(output_coin, vec![split_off]);

    let dest = SuiAddress::from_str(if quote.request.destination_address.is_empty() {
        &quote.request.wallet_address
    } else {
        &quote.request.destination_address
    })
    .map(Address::from)?;
    if dest == sender && is_sui_coin(route.output_coin_type()) {
        let gas = txb.gas();
        txb.merge_coins(gas, vec![output_coin]);
    } else {
        let dest_arg = txb.pure(&dest);
        txb.transfer_objects(vec![output_coin], dest_arg);
    }

    finish_transaction(txb, input.transaction.clone()).map_err(error)
}

fn sqrt_price_limit_with_slippage(hop: &Hop, slippage_bps: u32) -> u128 {
    if hop.after_sqrt_price == 0 {
        return if hop.a2b { MIN_SQRT_PRICE_X64 } else { MAX_SQRT_PRICE_X64 };
    }
    let bps = u128::from(slippage_bps);
    if hop.a2b {
        let limit = hop.after_sqrt_price.saturating_mul(10_000u128.saturating_sub(bps)) / 10_000;
        limit.max(MIN_SQRT_PRICE_X64)
    } else {
        let limit = hop.after_sqrt_price.saturating_mul(10_000u128.saturating_add(bps)).saturating_add(9_999) / 10_000;
        limit.min(MAX_SQRT_PRICE_X64)
    }
}

fn pay_referral_fee(txb: &mut TransactionBuilder, input_coin: Argument, fee: u64, referral_fee: &ReferralFee) -> Result<Argument, SwapperError> {
    if fee == 0 {
        return Ok(input_coin);
    }
    let fee_amount = txb.pure(&fee);
    let fee_coin = txb
        .split_coins(input_coin, vec![fee_amount])
        .pop()
        .ok_or_else(|| SwapperError::TransactionError("Sui referral fee split failed".into()))?;
    let recipient = SuiAddress::from_str(&referral_fee.address).map(Address::from)?;
    transfer_coin(txb, fee_coin, recipient);
    Ok(input_coin)
}

pub(super) fn referral_fee_amount(amount: u64, bps: u32) -> Result<u64, SwapperError> {
    amount
        .checked_mul(u64::from(bps))
        .map(|value| value / 10_000)
        .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus CLMM referral fee overflow".into()))
}

fn transfer_coin(txb: &mut TransactionBuilder, coin: Argument, recipient: Address) {
    let recipient = txb.pure(&recipient);
    txb.transfer_objects(vec![coin], recipient);
}

fn cetus_clmm_publish_at() -> Address {
    SuiAddress::from_str(CETUS_CLMM_PUBLISHED_AT).map(Address::from).unwrap()
}

fn shared_object_input(object_id: &str, initial_shared_version: u64, mutable: bool) -> Result<ObjectInput, SwapperError> {
    let address = SuiAddress::from_str(object_id).map(Address::from)?;
    Ok(ObjectInput::shared(address, initial_shared_version, mutable))
}

fn inspect_transaction_kind_bytes(mut txb: TransactionBuilder) -> Result<Vec<u8>, SwapperError> {
    txb.set_sender(SuiAddress::from_str(EMPTY_ADDRESS).map(Address::from)?);
    txb.set_gas_price(0);
    txb.set_gas_budget(0);
    txb.add_gas_objects(vec![ObjectInput::owned(Address::ZERO, 0, Digest::ZERO)]);
    let tx = txb.try_build().map_err(SwapperError::compute_quote_error)?;
    bcs::to_bytes(&tx.kind).map_err(SwapperError::compute_quote_error)
}

fn error(err: impl Display) -> SwapperError {
    SwapperError::transaction_error(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hop(a2b: bool, after_sqrt_price: u128) -> Hop {
        Hop {
            pool_id: "0xpool".into(),
            pool_init_version: 1,
            coin_a: "0xa".into(),
            coin_b: "0xb".into(),
            a2b,
            amount_in: 1_000,
            amount_out: 1_000_000,
            after_sqrt_price,
        }
    }

    #[test]
    fn test_sqrt_price_limit_with_slippage() {
        assert_eq!(sqrt_price_limit_with_slippage(&hop(true, 0), 50), MIN_SQRT_PRICE_X64);
        assert_eq!(sqrt_price_limit_with_slippage(&hop(false, 0), 50), MAX_SQRT_PRICE_X64);

        let after = 100_000_000_000_000u128;
        let a2b_limit = sqrt_price_limit_with_slippage(&hop(true, after), 50);
        assert_eq!(a2b_limit, after * 9_950 / 10_000);
        assert!(a2b_limit < after);

        let b2a_limit = sqrt_price_limit_with_slippage(&hop(false, after), 50);
        assert!(b2a_limit > after);
        assert!(b2a_limit <= MAX_SQRT_PRICE_X64);
    }

    #[test]
    fn test_referral_fee_amount() {
        assert_eq!(referral_fee_amount(1_000_000, 50).unwrap(), 5_000);
        assert_eq!(referral_fee_amount(1_000_000, 0).unwrap(), 0);
        assert!(referral_fee_amount(u64::MAX, 100).is_err());
    }
}
