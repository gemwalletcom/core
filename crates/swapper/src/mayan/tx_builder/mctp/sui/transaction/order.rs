use super::{
    add_publish_wormhole_message, deposit_for_burn_with_auth,
    fees::{bridge_amount, bridge_fee, redeem_fee},
    sui_error,
};
use crate::{
    Quote, SwapperError,
    mayan::{
        cctp_domain::domain_for_wormhole_chain,
        constants::{SUI_CCTP_CORE_STATE, SUI_MCTP_FEE_MANAGER_STATE, SUI_MCTP_STATE},
        model::{MayanMctpQuote, QuoteType},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{gas_drop_amount, min_amount_out, optional_bps_u8},
            swift::{referrer_bytes, swift_to_token},
        },
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
};
use gem_sui::{address::SuiAddress, tx_builder::move_call};
use sui_transaction_builder::{Argument, TransactionBuilder};
use sui_types::Address;

use super::super::prefetch::PrefetchedSuiData;

const MCTP_INIT_ORDER_PAYLOAD_ID: u8 = 1;

pub(super) fn add_init_order_move_calls(
    txb: &mut TransactionBuilder,
    route: &MayanMctpQuote,
    quote: &Quote,
    prefetched: &PrefetchedSuiData,
    input_coin: Argument,
    wh_fee_coin: Option<Argument>,
    destination_address: &str,
) -> Result<(), SwapperError> {
    let destination_chain_id = wormhole_chain_id(&route.to_chain)?;
    let mctp_input_contract = prefetched.mctp_input_contract.as_str();
    let fee_manager_package_id = prefetched.fee_manager_package_id.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let referrer = referrer_bytes(&route.to_chain)?;
    let amount_out_min = min_amount_out(&route.min_amount_out, route.to_token.decimals, &route.to_chain, &QuoteType::Mctp)?;
    let gas_drop = gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Mctp, false)?;
    let cctp_domain = domain_for_wormhole_chain(&route.to_chain)?.id();
    let fee_manager_package = Address::from(SuiAddress::parse(fee_manager_package_id).map_err(sui_error)?);
    let mctp_package = Address::from(SuiAddress::parse(&prefetched.mctp_package_id).map_err(sui_error)?);

    let common_arguments = vec![
        txb.object(prefetched.objects[&prefetched.mctp_verified_input_address].input(false)),
        txb.pure(&MCTP_INIT_ORDER_PAYLOAD_ID),
        txb.pure(&Address::from(SuiAddress::parse(&quote.request.wallet_address).map_err(sui_error)?)),
        input_coin,
        txb.pure(&Address::new(native_address_to_bytes32(destination_address, destination_chain_id)?)),
        txb.pure(&destination_chain_id),
        txb.pure(&Address::new(swift_to_token(route)?)),
        txb.pure(&amount_out_min),
        txb.pure(&gas_drop),
        txb.pure(&redeem_fee(route)?),
        txb.pure(&route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?),
        txb.pure(&Address::new(referrer)),
        txb.pure(&optional_bps_u8(route.referrer_bps)?),
    ];
    let fee_ticket = move_call(
        txb,
        fee_manager_package,
        "calculate_mctp_fee",
        "prepare_calc_mctp_fee",
        &[mctp_input_contract],
        common_arguments.clone(),
    )
    .map_err(sui_error)?;
    let fee_manager_state = txb.object(prefetched.objects[SUI_MCTP_FEE_MANAGER_STATE].input(false));
    let fee_params = move_call(
        txb,
        fee_manager_package,
        "calculate_mctp_fee",
        "calculate_mctp_fee",
        &[],
        vec![fee_manager_state, fee_ticket],
    )
    .map_err(sui_error)?;

    let mut init_arguments = common_arguments[1..].to_vec();
    init_arguments.push(txb.pure(&cctp_domain));
    init_arguments.push(txb.pure(&bridge_amount(route, mctp_input_contract)?));
    let init_ticket = move_call(txb, mctp_package, "init_order", "prepare_initialize_order", &[mctp_input_contract], init_arguments).map_err(sui_error)?;
    let mctp_state = txb.object(prefetched.objects[SUI_MCTP_STATE].input(true));
    let cctp_core_state = txb.object(prefetched.objects[SUI_CCTP_CORE_STATE].input(true));
    let verified_input = txb.object(prefetched.objects[&prefetched.mctp_verified_input_address].input(false));
    let initialize_result = move_call(
        txb,
        mctp_package,
        "init_order",
        "initialize_order",
        &[mctp_input_contract],
        vec![mctp_state, cctp_core_state, verified_input, init_ticket, fee_params],
    )
    .map_err(sui_error)?;
    let initialize_result = initialize_result.to_nested(2);
    let burn_request = initialize_result[0];
    let deposit_ticket = initialize_result[1];
    let cctp_message = deposit_for_burn_with_auth(txb, prefetched, mctp_input_contract, "init_order", deposit_ticket, &prefetched.mctp_input_treasury)?.to_nested(2)[1];
    let mctp_state = txb.object(prefetched.objects[SUI_MCTP_STATE].input(true));
    let wormhole_message = move_call(txb, mctp_package, "init_order", "publish_init_order", &[], vec![mctp_state, burn_request, cctp_message]).map_err(sui_error)?;
    add_publish_wormhole_message(txb, prefetched, wormhole_message, bridge_fee(route)?, wh_fee_coin)?;
    Ok(())
}
