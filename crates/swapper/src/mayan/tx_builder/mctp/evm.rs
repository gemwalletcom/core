use super::{destination_referrer_address, redeem_relayer_fee};
use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    mayan::{
        cctp_domain::{CCTP_TOKEN_DECIMALS, domain_for_wormhole_chain},
        client::MayanClient,
        model::{GetSwapEvmParams, GetSwapEvmResponse, MayanMctpQuote, QuoteType},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{fractional_amount, gas_decimals, gas_drop_amount, min_amount_out, optional_bps_u8},
            evm::{self as evm_builder, EvmForwarderProtocolCall, EvmSwapForwardData, EvmTransaction},
            route::quote_destination_address,
            swift::{referrer_bytes, swift_to_token},
        },
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_sol_types::{SolCall, sol};
use gem_client::Client;
use gem_evm::EVM_ZERO_ADDRESS;
use std::{fmt::Debug, str::FromStr, sync::Arc};

const MCTP_PAYLOAD_TYPE_DEFAULT: u8 = 1;

sol! {
    interface MayanCircle {
        struct OrderParams {
            address tokenIn;
            uint256 amountIn;
            uint64 gasDrop;
            bytes32 destAddr;
            uint16 destChain;
            bytes32 tokenOut;
            uint64 minAmountOut;
            uint64 deadline;
            uint64 redeemFee;
            bytes32 referrerAddr;
            uint8 referrerBps;
        }

        function bridgeWithFee(
            address tokenIn,
            uint256 amountIn,
            uint64 redeemFee,
            uint64 gasDrop,
            bytes32 destAddr,
            uint32 destDomain,
            uint8 payloadType,
            bytes customPayload
        ) external payable returns (uint64 sequence);
        function bridgeWithLockedFee(address tokenIn, uint256 amountIn, uint64 gasDrop, uint256 redeemFee, uint32 destDomain, bytes32 destAddr) external returns (uint64 cctpNonce);
        function createOrder(OrderParams params) external payable returns (uint64 sequence);
    }
}

fn mctp_protocol_call(quote: &Quote, route: &MayanMctpQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    if route.has_auction == Some(true) {
        mctp_create_order_call(quote, route)
    } else {
        mctp_bridge_call(quote, route)
    }
}

fn mctp_create_order_call(quote: &Quote, route: &MayanMctpQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    let contract_address = mctp_contract_address(route)?;
    let destination_chain_id = wormhole_chain_id(&route.to_chain)?;
    let destination_address = native_address_to_bytes32(quote_destination_address(quote), destination_chain_id)?;
    let amount_in = U256::from_str(&route.effective_amount_in64)?;
    let token_in = Address::from_str(mctp_input_contract(route)?)?;
    let referrer = referrer_bytes(&route.to_chain)?;
    let data = MayanCircle::createOrderCall {
        params: MayanCircle::OrderParams {
            tokenIn: token_in,
            amountIn: amount_in,
            gasDrop: gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Mctp, false)?,
            destAddr: FixedBytes::from(destination_address),
            destChain: destination_chain_id,
            tokenOut: FixedBytes::from(swift_to_token(route)?),
            minAmountOut: min_amount_out(&route.min_amount_out, route.to_token.decimals, &route.to_chain, &QuoteType::Mctp)?,
            deadline: route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?,
            redeemFee: redeem_relayer_fee(route)?,
            referrerAddr: FixedBytes::from(referrer),
            referrerBps: optional_bps_u8(route.referrer_bps)?,
        },
    }
    .abi_encode();
    Ok(EvmForwarderProtocolCall::new(amount_in, contract_address, data))
}

fn mctp_bridge_call(quote: &Quote, route: &MayanMctpQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    let contract_address = mctp_contract_address(route)?;
    let amount_in = U256::from_str(&route.effective_amount_in64)?;
    let token_in = Address::from_str(mctp_input_contract(route)?)?;
    let destination_chain_id = wormhole_chain_id(&route.to_chain)?;
    let destination_address = FixedBytes::from(native_address_to_bytes32(quote_destination_address(quote), destination_chain_id)?);
    let gas_drop = gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Mctp, false)?;
    let redeem_fee = redeem_relayer_fee(route)?;
    let destination_domain = domain_for_wormhole_chain(&route.to_chain)?.id();

    let data = if route.cheaper_chain.as_deref() == Some(route.from_chain.as_str()) {
        MayanCircle::bridgeWithLockedFeeCall {
            tokenIn: token_in,
            amountIn: amount_in,
            gasDrop: gas_drop,
            redeemFee: U256::from(redeem_fee),
            destDomain: destination_domain,
            destAddr: destination_address,
        }
        .abi_encode()
    } else {
        MayanCircle::bridgeWithFeeCall {
            tokenIn: token_in,
            amountIn: amount_in,
            redeemFee: redeem_fee,
            gasDrop: gas_drop,
            destAddr: destination_address,
            destDomain: destination_domain,
            payloadType: MCTP_PAYLOAD_TYPE_DEFAULT,
            customPayload: Bytes::new(),
        }
        .abi_encode()
    };

    Ok(EvmForwarderProtocolCall::new(amount_in, contract_address, data))
}

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanMctpQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    evm_builder::build_quote_data(build(client, quote, route), quote, rpc_provider).await
}

async fn build<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanMctpQuote) -> Result<EvmTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let protocol_call = mctp_protocol_call(quote, route)?;
    let bridge_fee = bridge_fee(route)?;
    if route.from_token.contract.eq_ignore_ascii_case(mctp_input_contract(route)?) {
        return build_direct_forward_transaction(route, &protocol_call, bridge_fee);
    }

    build_swap_forward_transaction(client, route, &protocol_call, bridge_fee).await
}

fn build_direct_forward_transaction(route: &MayanMctpQuote, protocol_call: &EvmForwarderProtocolCall, bridge_fee: U256) -> Result<EvmTransaction, SwapperError> {
    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Err(SwapperError::transaction_error("Mayan MCTP does not support direct native order creation"));
    }

    Ok(evm_builder::build_forward_erc20_transaction(
        Address::from_str(&route.from_token.contract)?,
        protocol_call,
        bridge_fee.to_string(),
    ))
}

async fn build_swap_forward_transaction<C>(
    client: &MayanClient<C>,
    route: &MayanMctpQuote,
    protocol_call: &EvmForwarderProtocolCall,
    bridge_fee: U256,
) -> Result<EvmTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let mctp_input_contract = mctp_input_contract(route)?;
    let min_middle_amount = fractional_amount::<U256>(route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)?;
    let swap: GetSwapEvmResponse = client
        .get_swap(
            "/get-swap/evm",
            GetSwapEvmParams::mctp(
                route,
                route.effective_amount_in64.clone(),
                mctp_input_contract.to_string(),
                destination_referrer_address(route)?,
            ),
        )
        .await?;
    let swap = EvmSwapForwardData::new(&swap.swap_router_address, &swap.swap_router_calldata, mctp_input_contract, min_middle_amount)?;

    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        let amount_in = protocol_call
            .amount_in
            .checked_sub(bridge_fee)
            .ok_or_else(|| SwapperError::transaction_error("Amount in is less than bridge fee"))?;
        return Ok(evm_builder::build_swap_and_forward_eth_transaction(
            protocol_call,
            swap,
            amount_in,
            protocol_call.amount_in.to_string(),
        ));
    }

    Ok(evm_builder::build_swap_and_forward_erc20_transaction(
        Address::from_str(&route.from_token.contract)?,
        protocol_call,
        swap,
        bridge_fee.to_string(),
    ))
}

fn mctp_contract_address(route: &MayanMctpQuote) -> Result<Address, SwapperError> {
    Address::from_str(route.mctp_mayan_contract.as_deref().ok_or(SwapperError::InvalidRoute)?).map_err(SwapperError::from)
}

fn mctp_input_contract(route: &MayanMctpQuote) -> Result<&str, SwapperError> {
    route.mctp_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn bridge_fee(route: &MayanMctpQuote) -> Result<U256, SwapperError> {
    fractional_amount(route.bridge_fee.as_ref().ok_or(SwapperError::InvalidRoute)?, gas_decimals(&route.from_chain)?)
}
