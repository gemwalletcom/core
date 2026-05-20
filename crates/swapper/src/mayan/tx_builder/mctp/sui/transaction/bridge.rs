use super::{
    add_publish_wormhole_message, deposit_for_burn_with_auth,
    fees::{bridge_amount, bridge_fee, redeem_fee},
};
use crate::{
    SwapperError,
    mayan::{
        cctp_domain::domain_for_wormhole_chain,
        constants::{SUI_CCTP_CORE_STATE, SUI_MCTP_STATE},
        model::{MayanMctpQuote, QuoteType},
        tx_builder::{address::native_address_to_bytes32, amount::gas_drop_amount},
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
};
use gem_sui::{address::SuiAddress, tx_builder::move_call};
use sui_transaction_builder::{Argument, TransactionBuilder};
use sui_types::Address;

use super::super::prefetch::PrefetchedSuiData;
use super::sui_error;

const MCTP_PAYLOAD_TYPE_DEFAULT: u8 = 1;
const BRIDGE_WITH_FEE_MODULE: &str = "bridge_with_fee";
const BRIDGE_LOCKED_FEE_MODULE: &str = "bridge_locked_fee";

struct BridgeParams {
    amount_in: u64,
    destination: Address,
    domain: u32,
    gas_drop: u64,
    redeem_fee: u64,
}

impl BridgeParams {
    fn new(route: &MayanMctpQuote, mctp_input_contract: &str, destination_address: &str) -> Result<Self, SwapperError> {
        let destination_chain_id = wormhole_chain_id(&route.to_chain)?;
        Ok(Self {
            amount_in: bridge_amount(route, mctp_input_contract)?,
            destination: Address::new(native_address_to_bytes32(destination_address, destination_chain_id)?),
            domain: domain_for_wormhole_chain(&route.to_chain)?.id(),
            gas_drop: gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Mctp, false)?,
            redeem_fee: redeem_fee(route)?,
        })
    }
}

pub(super) fn add_bridge_with_fee_move_calls(
    txb: &mut TransactionBuilder,
    route: &MayanMctpQuote,
    prefetched: &PrefetchedSuiData,
    input_coin: Argument,
    wh_fee_coin: Option<Argument>,
    destination_address: &str,
) -> Result<(), SwapperError> {
    let mctp_input_contract = prefetched.mctp_input_contract.as_str();
    let bridge = BridgeParams::new(route, mctp_input_contract, destination_address)?;
    let payload = Vec::<u8>::new();
    let mctp_package = mctp_package_address(prefetched)?;
    let bridge_arguments = vec![
        txb.pure(&MCTP_PAYLOAD_TYPE_DEFAULT),
        input_coin,
        txb.pure(&bridge.amount_in),
        txb.pure(&bridge.destination),
        txb.pure(&bridge.domain),
        txb.pure(&bridge.gas_drop),
        txb.pure(&bridge.redeem_fee),
        txb.pure(&payload),
    ];
    let bridge_ticket = move_call(
        txb,
        mctp_package,
        BRIDGE_WITH_FEE_MODULE,
        "prepare_bridge_with_fee",
        &[mctp_input_contract],
        bridge_arguments,
    )
    .map_err(sui_error)?;
    let (burn_request, cctp_message) = complete_bridge(txb, prefetched, mctp_input_contract, BRIDGE_WITH_FEE_MODULE, bridge_ticket)?;
    let mctp_state = txb.object(prefetched.objects[SUI_MCTP_STATE].input(true));
    let wormhole_message = move_call(
        txb,
        mctp_package,
        BRIDGE_WITH_FEE_MODULE,
        "publish_bridge_with_fee",
        &[],
        vec![mctp_state, burn_request, cctp_message],
    )
    .map_err(sui_error)?;
    add_publish_wormhole_message(txb, prefetched, wormhole_message, bridge_fee(route)?, wh_fee_coin)?;
    Ok(())
}

pub(super) fn add_bridge_locked_fee_move_calls(
    txb: &mut TransactionBuilder,
    route: &MayanMctpQuote,
    prefetched: &PrefetchedSuiData,
    input_coin: Argument,
    destination_address: &str,
) -> Result<(), SwapperError> {
    let mctp_input_contract = prefetched.mctp_input_contract.as_str();
    let bridge = BridgeParams::new(route, mctp_input_contract, destination_address)?;
    let mctp_package = mctp_package_address(prefetched)?;
    let bridge_arguments = vec![
        input_coin,
        txb.pure(&bridge.amount_in),
        txb.pure(&bridge.destination),
        txb.pure(&bridge.domain),
        txb.pure(&bridge.gas_drop),
        txb.pure(&bridge.redeem_fee),
    ];
    let bridge_ticket = move_call(
        txb,
        mctp_package,
        BRIDGE_LOCKED_FEE_MODULE,
        "prepare_bridge_locked_fee",
        &[mctp_input_contract],
        bridge_arguments,
    )
    .map_err(sui_error)?;
    let (burn_request, cctp_message) = complete_bridge(txb, prefetched, mctp_input_contract, BRIDGE_LOCKED_FEE_MODULE, bridge_ticket)?;
    let mctp_state = txb.object(prefetched.objects[SUI_MCTP_STATE].input(true));
    let verified_input = txb.object(prefetched.objects[&prefetched.mctp_verified_input_address].input(false));
    move_call(
        txb,
        mctp_package,
        BRIDGE_LOCKED_FEE_MODULE,
        "store_bridge_locked_fee",
        &[mctp_input_contract],
        vec![mctp_state, verified_input, burn_request, cctp_message],
    )
    .map_err(sui_error)?;
    Ok(())
}

fn complete_bridge(
    txb: &mut TransactionBuilder,
    prefetched: &PrefetchedSuiData,
    mctp_input_contract: &str,
    module: &str,
    bridge_ticket: Argument,
) -> Result<(Argument, Argument), SwapperError> {
    let mctp_package = mctp_package_address(prefetched)?;
    let mctp_state = txb.object(prefetched.objects[SUI_MCTP_STATE].input(true));
    let cctp_core_state = txb.object(prefetched.objects[SUI_CCTP_CORE_STATE].input(true));
    let verified_input = txb.object(prefetched.objects[&prefetched.mctp_verified_input_address].input(false));
    let bridge_result = move_call(
        txb,
        mctp_package,
        module,
        module,
        &[mctp_input_contract],
        vec![mctp_state, cctp_core_state, verified_input, bridge_ticket],
    )
    .map_err(sui_error)?;
    let bridge_result = bridge_result.to_nested(2);
    let burn_request = bridge_result[0];
    let deposit_ticket = bridge_result[1];
    let cctp_message = deposit_for_burn_with_auth(txb, prefetched, mctp_input_contract, module, deposit_ticket, &prefetched.mctp_input_treasury)?.to_nested(2)[1];
    Ok((burn_request, cctp_message))
}

fn mctp_package_address(prefetched: &PrefetchedSuiData) -> Result<Address, SwapperError> {
    SuiAddress::parse(&prefetched.mctp_package_id).map(Address::from).map_err(sui_error)
}
