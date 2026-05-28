mod bridge;
mod fees;
mod order;

use self::{
    bridge::{add_bridge_locked_fee_move_calls, add_bridge_with_fee_move_calls},
    order::add_init_order_move_calls,
};
use super::{prefetch::PrefetchedSuiData, sui_error};
use crate::mayan::{
    constants::{SUI_CCTP_CORE_STATE, SUI_CCTP_DENY_LIST, SUI_CCTP_TOKEN_PACKAGE_ID, SUI_CCTP_TOKEN_STATE, SUI_LOGGER_PACKAGE_ID, SUI_WORMHOLE_PACKAGE_ID, SUI_WORMHOLE_STATE},
    model::{MayanMctpQuote, SuiClientSwap},
    wormhole_chain::WormholeChain,
};
use crate::{
    Quote, SwapperError,
    mayan::tx_builder::{amount::optional_bps_u8, swift::referrer_bytes},
};
use gem_sui::{
    SUI_COIN_TYPE,
    address::SuiAddress,
    models::{OwnedCoins, TxOutput},
    sui_clock_object_input,
    tx_builder::{TransactionJsonReplay, build_input_coin, finish_transaction, move_call},
};
use sui_transaction_builder::{Argument, TransactionBuilder};
use sui_types::Address;

#[cfg(test)]
pub(super) use fees::bridge_amount;

pub(super) fn build_mctp_transaction(
    quote: &Quote,
    route: &MayanMctpQuote,
    prefetched: &PrefetchedSuiData,
    destination_address: &str,
    swap: Option<&SuiClientSwap>,
    swap_replay: Option<&TransactionJsonReplay>,
    gas_budget: u64,
) -> Result<TxOutput, SwapperError> {
    let (mut txb, input_coin, wh_fee_coin) = input_coin(route, prefetched, swap, swap_replay)?;

    if route.has_auction == Some(true) {
        add_init_order_move_calls(&mut txb, route, quote, prefetched, input_coin, wh_fee_coin, destination_address)?;
    } else if route.cheaper_chain.as_deref() == Some(WormholeChain::Sui.name()) {
        add_bridge_locked_fee_move_calls(&mut txb, route, prefetched, input_coin, destination_address)?;
    } else {
        add_bridge_with_fee_move_calls(&mut txb, route, prefetched, input_coin, wh_fee_coin, destination_address)?;
    }

    log_initialize_mctp(&mut txb, route, prefetched)?;
    log_referrer(&mut txb, route)?;

    finish_transaction(txb, prefetched.transaction.with_gas_budget(gas_budget)).map_err(sui_error)
}

fn input_coin(
    route: &MayanMctpQuote,
    prefetched: &PrefetchedSuiData,
    swap: Option<&SuiClientSwap>,
    swap_replay: Option<&TransactionJsonReplay>,
) -> Result<(TransactionBuilder, Argument, Option<Argument>), SwapperError> {
    if let Some(swap) = swap {
        let replayed = swap_replay.ok_or(SwapperError::InvalidRoute)?.replay().map_err(sui_error)?;
        let input_coin = replayed.argument(&swap.out_coin).map_err(sui_error)?;
        let wh_fee_coin = swap.wh_fee_coin.as_ref().map(|argument| replayed.argument(argument).map_err(sui_error)).transpose()?;
        return Ok((replayed.txb, input_coin, wh_fee_coin));
    }

    let mut txb = TransactionBuilder::new();
    let input_coin = build_input_coin(
        &mut txb,
        &prefetched.mctp_input_contract,
        route.effective_amount_in64.parse::<u64>()?,
        &prefetched.input_coins,
    )
    .map_err(sui_error)?;
    Ok((txb, input_coin, None))
}

fn deposit_for_burn_with_auth(
    txb: &mut TransactionBuilder,
    prefetched: &PrefetchedSuiData,
    mctp_input_contract: &str,
    auth_module: &str,
    deposit_ticket: Argument,
    treasury: &str,
) -> Result<Argument, SwapperError> {
    let cctp_token_state = txb.object(prefetched.objects[SUI_CCTP_TOKEN_STATE].input(true));
    let cctp_core_state = txb.object(prefetched.objects[SUI_CCTP_CORE_STATE].input(true));
    let deny_list = txb.object(prefetched.objects[SUI_CCTP_DENY_LIST].input(false));
    let treasury = txb.object(prefetched.objects[treasury].input(true));
    let auth_type = format!("{}::{auth_module}::CircleAuth", prefetched.mctp_package_id);
    let package = SuiAddress::parse(SUI_CCTP_TOKEN_PACKAGE_ID).map_err(sui_error)?.into();
    move_call(
        txb,
        package,
        "deposit_for_burn",
        "deposit_for_burn_with_caller_with_package_auth",
        &[mctp_input_contract, &auth_type],
        vec![deposit_ticket, cctp_token_state, cctp_core_state, deny_list, treasury],
    )
    .map_err(sui_error)
}

fn add_publish_wormhole_message(
    txb: &mut TransactionBuilder,
    prefetched: &PrefetchedSuiData,
    wormhole_message: Argument,
    bridge_fee: u64,
    wh_fee_coin: Option<Argument>,
) -> Result<(), SwapperError> {
    let fee_coin = match wh_fee_coin {
        Some(coin) => coin,
        None => build_input_coin(txb, SUI_COIN_TYPE, bridge_fee, &OwnedCoins::default()).map_err(sui_error)?,
    };
    let clock = txb.object(sui_clock_object_input());
    let wormhole_state = txb.object(prefetched.objects[SUI_WORMHOLE_STATE].input(true));
    let package = SuiAddress::parse(SUI_WORMHOLE_PACKAGE_ID).map_err(sui_error)?.into();
    move_call(
        txb,
        package,
        "publish_message",
        "publish_message",
        &[],
        vec![wormhole_state, fee_coin, wormhole_message, clock],
    )
    .map_err(sui_error)?;
    Ok(())
}

fn log_initialize_mctp(txb: &mut TransactionBuilder, route: &MayanMctpQuote, prefetched: &PrefetchedSuiData) -> Result<(), SwapperError> {
    let amount = route.effective_amount_in64.parse::<u64>()?;
    let verified_input = txb.object(prefetched.objects[&prefetched.from_token_verified_address].input(false));
    let amount = txb.pure(&amount);
    let payload = txb.pure(&Vec::<u8>::new());
    let package = SuiAddress::parse(&prefetched.mctp_package_id).map_err(sui_error)?.into();
    move_call(
        txb,
        package,
        "init_order",
        "log_initialize_mctp",
        &[&route.from_token.contract],
        vec![amount, verified_input, payload],
    )
    .map_err(sui_error)?;
    Ok(())
}

fn log_referrer(txb: &mut TransactionBuilder, route: &MayanMctpQuote) -> Result<(), SwapperError> {
    let referrer = referrer_bytes(&route.to_chain)?;
    let referrer = txb.pure(&Address::new(referrer));
    let referrer_bps = txb.pure(&optional_bps_u8(route.referrer_bps)?);
    let package = SuiAddress::parse(SUI_LOGGER_PACKAGE_ID).map_err(sui_error)?.into();
    move_call(txb, package, "referrer_logger", "log_referrer", &[], vec![referrer, referrer_bps]).map_err(sui_error)?;
    Ok(())
}
