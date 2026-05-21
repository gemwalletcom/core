use super::{contracts::MayanSwiftV2, order::swift_order};
use crate::{
    Quote, SwapperError,
    mayan::{
        client::MayanClient,
        model::{GetSwapEvmParams, GetSwapEvmResponse, MayanSwiftQuote},
        tx_builder::{
            amount::fractional_amount,
            evm::{EvmTransaction, MayanForwarder},
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
use primitives::decode_hex;
use std::{fmt::Debug, str::FromStr};

struct EvmSwiftContext {
    amount_in: U256,
    swift_input_contract: String,
    swift_contract_address: Address,
    swift_token_in: Address,
    swift_call_data: Vec<u8>,
}

impl EvmSwiftContext {
    fn new(quote: &Quote, route: &MayanSwiftQuote) -> Result<Self, SwapperError> {
        let source_chain_id = wormhole_chain_id(&route.from_chain)?;
        let amount_in = U256::from_str(&quote.from_value)?;
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
        let swift_call_data = MayanSwiftV2::createOrderWithTokenCall {
            tokenIn: swift_token_in,
            amountIn: amount_in,
            params: order,
            customPayload: Bytes::from(custom_payload.unwrap_or_default()),
        }
        .abi_encode();

        Ok(Self {
            amount_in,
            swift_input_contract,
            swift_contract_address,
            swift_token_in,
            swift_call_data,
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
        build_swap_forward_transaction(client, quote, route, &context).await
    }
}

fn build_direct_forward_transaction(route: &MayanSwiftQuote, context: &EvmSwiftContext) -> Result<EvmTransaction, SwapperError> {
    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Err(SwapperError::transaction_error("Mayan Swift V2 does not support direct native order creation"));
    }

    let data = MayanForwarder::forwardERC20Call {
        tokenIn: context.swift_token_in,
        amountIn: context.amount_in,
        permitParams: MayanForwarder::PermitParams::default(),
        mayanProtocol: context.swift_contract_address,
        protocolData: Bytes::from(context.swift_call_data.clone()),
    }
    .abi_encode();
    Ok(EvmTransaction::forwarder("0", data))
}

async fn build_swap_forward_transaction<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanSwiftQuote, context: &EvmSwiftContext) -> Result<EvmTransaction, SwapperError>
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
            GetSwapEvmParams::swift(route, quote.from_value.clone(), context.swift_input_contract.clone()),
        )
        .await?;
    let swap_router_address = Address::from_str(&swap.swap_router_address)?;
    let swap_router_calldata = Bytes::from(decode_hex(&swap.swap_router_calldata)?);
    let middle_token = Address::from_str(&context.swift_input_contract)?;

    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        let data = MayanForwarder::swapAndForwardEthCall {
            amountIn: context.amount_in,
            swapProtocol: swap_router_address,
            swapData: swap_router_calldata,
            middleToken: middle_token,
            minMiddleAmount: min_middle_amount,
            mayanProtocol: context.swift_contract_address,
            mayanData: Bytes::from(context.swift_call_data.clone()),
        }
        .abi_encode();
        return Ok(EvmTransaction::forwarder(context.amount_in.to_string(), data));
    }

    let data = MayanForwarder::swapAndForwardERC20Call {
        tokenIn: Address::from_str(&route.from_token.contract)?,
        amountIn: context.amount_in,
        permitParams: MayanForwarder::PermitParams::default(),
        swapProtocol: swap_router_address,
        swapData: swap_router_calldata,
        middleToken: middle_token,
        minMiddleAmount: min_middle_amount,
        mayanProtocol: context.swift_contract_address,
        mayanData: Bytes::from(context.swift_call_data.clone()),
    }
    .abi_encode();
    Ok(EvmTransaction::forwarder("0", data))
}
