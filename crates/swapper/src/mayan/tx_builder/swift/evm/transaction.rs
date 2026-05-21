use super::{contracts::MayanSwiftV2, order::swift_order};
use crate::{
    Quote, SwapperError,
    mayan::{
        client::MayanClient,
        model::{GetSwapEvmParams, GetSwapEvmResponse, MayanSwiftQuote},
        tx_builder::{
            amount::fractional_amount,
            evm::{self as evm_builder, EvmForwarderProtocolCall, EvmSwapForwardData, EvmTransaction},
            hypercore::hypercore_custom_payload,
            route::{quote_destination_address, swift_destination_address},
            swift::swift_input_contract as route_swift_input_contract,
        },
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::SolCall;
use gem_client::Client;
use gem_evm::EVM_ZERO_ADDRESS;
use std::{fmt::Debug, str::FromStr};

struct EvmSwiftContext {
    swift_input_contract: String,
    swift_token_in: Address,
    protocol_call: EvmForwarderProtocolCall,
}

impl EvmSwiftContext {
    fn new(quote: &Quote, route: &MayanSwiftQuote) -> Result<Self, SwapperError> {
        let source_chain_id = wormhole_chain_id(&route.from_chain)?;
        let amount_in = U256::from_str(&route.effective_amount_in64)?;
        let swift_input_contract = route_swift_input_contract(route)?.to_string();
        let swift_contract_address = Address::from_str(route.swift_mayan_contract.as_deref().ok_or(SwapperError::InvalidRoute)?)?;
        let swift_token_in = if route.swift_wrap_and_lock == Some(true) {
            Address::ZERO
        } else {
            Address::from_str(&swift_input_contract)?
        };
        let destination_address = swift_destination_address(quote, route);
        let custom_payload = hypercore_custom_payload(route, quote_destination_address(quote))?;
        let order = swift_order(quote, route, source_chain_id, destination_address.as_ref(), custom_payload.as_deref())?;
        let data = MayanSwiftV2::createOrderWithTokenCall {
            tokenIn: swift_token_in,
            amountIn: amount_in,
            params: order,
            customPayload: Bytes::from(custom_payload.unwrap_or_default()),
        }
        .abi_encode();

        Ok(Self {
            swift_input_contract,
            swift_token_in,
            protocol_call: EvmForwarderProtocolCall::new(amount_in, swift_contract_address, data),
        })
    }
}

pub(super) async fn build<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanSwiftQuote) -> Result<EvmTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let context = EvmSwiftContext::new(quote, route)?;
    if route.from_token.contract.eq_ignore_ascii_case(&context.swift_input_contract) {
        build_direct_forward_transaction(route, &context)
    } else {
        build_swap_forward_transaction(client, route, &context).await
    }
}

fn build_direct_forward_transaction(route: &MayanSwiftQuote, context: &EvmSwiftContext) -> Result<EvmTransaction, SwapperError> {
    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Err(SwapperError::transaction_error("Mayan Swift V2 does not support direct native order creation"));
    }

    Ok(evm_builder::build_forward_erc20_transaction(context.swift_token_in, &context.protocol_call, "0"))
}

async fn build_swap_forward_transaction<C>(client: &MayanClient<C>, route: &MayanSwiftQuote, context: &EvmSwiftContext) -> Result<EvmTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let min_middle_amount = fractional_amount::<U256>(
        route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?,
        route.swift_input_decimals.ok_or(SwapperError::InvalidRoute)?,
    )?;
    let swap: GetSwapEvmResponse = client
        .get_swap(
            "/get-swap/evm",
            GetSwapEvmParams::swift(route, route.effective_amount_in64.clone(), context.swift_input_contract.clone()),
        )
        .await?;
    let swap = EvmSwapForwardData::new(&swap.swap_router_address, &swap.swap_router_calldata, &context.swift_input_contract, min_middle_amount)?;

    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Ok(evm_builder::build_swap_and_forward_eth_transaction(
            &context.protocol_call,
            swap,
            context.protocol_call.amount_in,
            context.protocol_call.amount_in.to_string(),
        ));
    }

    Ok(evm_builder::build_swap_and_forward_erc20_transaction(
        Address::from_str(&route.from_token.contract)?,
        &context.protocol_call,
        swap,
        "0",
    ))
}
