use super::{
    FAST_MCTP_PAYLOAD_TYPE_DEFAULT, FAST_MCTP_PAYLOAD_TYPE_ORDER, circle_max_fee64, destination_referrer_address, fast_mctp_contract, fast_mctp_input_contract,
    fast_mctp_min_finality, redeem_relayer_fee, referrer_bytes, refund_relayer_fee64, token_out,
};
use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    mayan::{
        cctp_domain::{CCTP_TOKEN_DECIMALS, domain_for_wormhole_chain},
        client::MayanClient,
        model::{GetSwapEvmParams, GetSwapEvmResponse, MayanFastMctpQuote, QuoteType},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{fractional_amount, gas_drop_amount, min_amount_out, optional_bps_u8},
            evm::{self as evm_builder, EvmForwarderProtocolCall, EvmSwapForwardData, EvmTransaction},
            route::quote_destination_address,
        },
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_sol_types::{SolCall, sol};
use gem_client::Client;
use gem_evm::EVM_ZERO_ADDRESS;
use std::{fmt::Debug, str::FromStr, sync::Arc};

sol! {
    interface MayanFastMctp {
        struct OrderPayload {
            uint8 payloadType;
            bytes32 destAddr;
            bytes32 tokenOut;
            uint64 amountOutMin;
            uint64 gasDrop;
            uint64 redeemFee;
            uint64 refundFee;
            uint64 deadline;
            bytes32 referrerAddr;
            uint8 referrerBps;
        }

        function bridge(
            address tokenIn,
            uint256 amountIn,
            uint64 redeemFee,
            uint256 circleMaxFee,
            uint64 gasDrop,
            bytes32 destAddr,
            uint32 destDomain,
            bytes32 referrerAddress,
            uint8 referrerBps,
            uint8 payloadType,
            uint32 minFinalityThreshold,
            bytes customPayload
        ) external;
        function createOrder(address tokenIn, uint256 amountIn, uint256 circleMaxFee, uint32 destDomain, uint32 minFinalityThreshold, OrderPayload orderPayload) external;
    }
}

fn fast_mctp_protocol_call(quote: &Quote, route: &MayanFastMctpQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    if route.has_auction == Some(true) {
        fast_mctp_create_order_call(quote, route)
    } else {
        fast_mctp_bridge_call(quote, route)
    }
}

fn fast_mctp_create_order_call(quote: &Quote, route: &MayanFastMctpQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    let contract_address = Address::from_str(fast_mctp_contract(route)?)?;
    let destination_chain_id = wormhole_chain_id(&route.to_chain)?;
    let destination_address = FixedBytes::from(native_address_to_bytes32(quote_destination_address(quote), destination_chain_id)?);
    let amount_in = U256::from_str(&route.effective_amount_in64)?;
    let token_in = Address::from_str(fast_mctp_input_contract(route)?)?;
    let circle_max_fee = U256::from_str(circle_max_fee64(route)?)?;
    let data = MayanFastMctp::createOrderCall {
        tokenIn: token_in,
        amountIn: amount_in,
        circleMaxFee: circle_max_fee,
        destDomain: domain_for_wormhole_chain(&route.to_chain)?.id(),
        minFinalityThreshold: fast_mctp_min_finality(route)?,
        orderPayload: MayanFastMctp::OrderPayload {
            payloadType: FAST_MCTP_PAYLOAD_TYPE_ORDER,
            destAddr: destination_address,
            tokenOut: FixedBytes::from(token_out(route)?),
            amountOutMin: min_amount_out(&route.min_amount_out, route.to_token.decimals, &route.to_chain, &QuoteType::FastMctp)?,
            gasDrop: gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::FastMctp, false)?,
            redeemFee: redeem_relayer_fee(route)?,
            refundFee: refund_relayer_fee64(route)?,
            deadline: route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?,
            referrerAddr: FixedBytes::from(referrer_bytes(route)?),
            referrerBps: optional_bps_u8(route.referrer_bps)?,
        },
    }
    .abi_encode();
    Ok(EvmForwarderProtocolCall::new(amount_in, contract_address, data))
}

fn fast_mctp_bridge_call(quote: &Quote, route: &MayanFastMctpQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    let contract_address = Address::from_str(fast_mctp_contract(route)?)?;
    let destination_chain_id = wormhole_chain_id(&route.to_chain)?;
    let amount_in = U256::from_str(&route.effective_amount_in64)?;
    let token_in = Address::from_str(fast_mctp_input_contract(route)?)?;
    let data = MayanFastMctp::bridgeCall {
        tokenIn: token_in,
        amountIn: amount_in,
        redeemFee: redeem_relayer_fee(route)?,
        circleMaxFee: U256::from_str(circle_max_fee64(route)?)?,
        gasDrop: gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::FastMctp, false)?,
        destAddr: FixedBytes::from(native_address_to_bytes32(quote_destination_address(quote), destination_chain_id)?),
        destDomain: domain_for_wormhole_chain(&route.to_chain)?.id(),
        referrerAddress: FixedBytes::from(referrer_bytes(route)?),
        referrerBps: optional_bps_u8(route.referrer_bps)?,
        payloadType: FAST_MCTP_PAYLOAD_TYPE_DEFAULT,
        minFinalityThreshold: fast_mctp_min_finality(route)?,
        customPayload: Bytes::new(),
    }
    .abi_encode();
    Ok(EvmForwarderProtocolCall::new(amount_in, contract_address, data))
}

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanFastMctpQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    evm_builder::build_quote_data(build(client, quote, route), quote, rpc_provider).await
}

async fn build<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanFastMctpQuote) -> Result<EvmTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let protocol_call = fast_mctp_protocol_call(quote, route)?;
    if route.from_token.contract.eq_ignore_ascii_case(fast_mctp_input_contract(route)?) {
        return build_direct_forward_transaction(route, &protocol_call);
    }

    build_swap_forward_transaction(client, route, &protocol_call).await
}

fn build_direct_forward_transaction(route: &MayanFastMctpQuote, protocol_call: &EvmForwarderProtocolCall) -> Result<EvmTransaction, SwapperError> {
    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Err(SwapperError::transaction_error("Mayan FastMCTP does not support direct native order creation"));
    }

    Ok(evm_builder::build_forward_erc20_transaction(
        Address::from_str(&route.from_token.contract)?,
        protocol_call,
        "0",
    ))
}

async fn build_swap_forward_transaction<C>(client: &MayanClient<C>, route: &MayanFastMctpQuote, protocol_call: &EvmForwarderProtocolCall) -> Result<EvmTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let fast_mctp_input_contract = fast_mctp_input_contract(route)?;
    let min_middle_amount = fractional_amount::<U256>(route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)?;
    let swap: GetSwapEvmResponse = client
        .get_swap(
            "/get-swap/evm",
            GetSwapEvmParams::fast_mctp(
                route,
                route.effective_amount_in64.clone(),
                fast_mctp_input_contract.to_string(),
                destination_referrer_address(route)?,
            ),
        )
        .await?;
    let swap = EvmSwapForwardData::new(&swap.swap_router_address, &swap.swap_router_calldata, fast_mctp_input_contract, min_middle_amount)?;

    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Ok(evm_builder::build_swap_and_forward_eth_transaction(
            protocol_call,
            swap,
            protocol_call.amount_in,
            protocol_call.amount_in.to_string(),
        ));
    }

    Ok(evm_builder::build_swap_and_forward_erc20_transaction(
        Address::from_str(&route.from_token.contract)?,
        protocol_call,
        swap,
        "0",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, asset_constants::ETHEREUM_USDC_TOKEN_ID, hex};

    #[test]
    fn test_build_direct_forward_transaction_wraps_fast_mctp_call() {
        let mut quote = Quote::mock(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID));
        quote.request.wallet_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string();
        quote.request.destination_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string();
        let route = MayanFastMctpQuote::mock_evm();
        let protocol_call = fast_mctp_protocol_call(&quote, &route).unwrap();
        let transaction = build_direct_forward_transaction(&route, &protocol_call).unwrap();

        assert_eq!(transaction.value, "0");
        assert_eq!(transaction.data[..10], hex::encode_with_0x(&evm_builder::MayanForwarder::forwardERC20Call::SELECTOR));
    }
}
