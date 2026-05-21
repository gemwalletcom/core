use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    fees::default_referral_address,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        model::MayanMonoChainQuote,
        tx_builder::{
            amount::fractional_amount,
            evm::{self as evm_builder, EvmForwarderProtocolCall, EvmSwapForwardData, EvmTransaction},
            hypercore::hypercore_deposit_dex,
            route::quote_destination_address,
        },
        wormhole_chain::WormholeChain,
    },
};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{SolCall, sol};
use gem_evm::EVM_ZERO_ADDRESS;
use primitives::asset_constants::HYPEREVM_USDC_TOKEN_ID;
use std::{str::FromStr, sync::Arc};

const MAX_BPS: u32 = 10_000;

sol! {
    interface MayanHyperCoreDeposit {
        function depositToHyperCore(address tokenIn, uint256 amountIn, uint16 referrerBps, address referrerAddr, address destAddr, uint32 destDex) external;
    }
}

fn hypercore_deposit_call(quote: &Quote, route: &MayanMonoChainQuote) -> Result<EvmForwarderProtocolCall, SwapperError> {
    if route.from_chain != WormholeChain::Hyperevm.name() || route.to_chain != WormholeChain::Hypercore.name() {
        return Err(SwapperError::InvalidRoute);
    }

    let contract_address = Address::from_str(&route.mono_chain_mayan_contract)?;
    let amount_in = U256::from_str(&route.effective_amount_in64)?;
    let referrer_address = default_referral_address(quote.request.from_asset.chain());
    let has_referrer = !referrer_address.is_empty();
    let referrer_bps = match route.referrer_bps {
        Some(value) if value <= MAX_BPS => value as u16,
        Some(_) => return Err(SwapperError::InvalidRoute),
        None => 0,
    };
    let data = MayanHyperCoreDeposit::depositToHyperCoreCall {
        tokenIn: Address::from_str(HYPEREVM_USDC_TOKEN_ID)?,
        amountIn: amount_in,
        referrerBps: if has_referrer { referrer_bps } else { 0 },
        referrerAddr: if has_referrer { Address::from_str(&referrer_address)? } else { Address::ZERO },
        destAddr: Address::from_str(quote_destination_address(quote))?,
        destDex: hypercore_deposit_dex(&route.to_token.contract)?,
    }
    .abi_encode();

    Ok(EvmForwarderProtocolCall::new(amount_in, contract_address, data))
}

pub async fn build_quote_data(quote: &Quote, route: &MayanMonoChainQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError> {
    evm_builder::build_quote_data(build(quote, route), quote, rpc_provider).await
}

async fn build(quote: &Quote, route: &MayanMonoChainQuote) -> Result<EvmTransaction, SwapperError> {
    let deposit_call = hypercore_deposit_call(quote, route)?;
    if route.from_token.contract.eq_ignore_ascii_case(HYPEREVM_USDC_TOKEN_ID) {
        return build_direct_forward_transaction(&deposit_call);
    }

    build_swap_forward_transaction(route, &deposit_call)
}

fn build_direct_forward_transaction(deposit_call: &EvmForwarderProtocolCall) -> Result<EvmTransaction, SwapperError> {
    Ok(evm_builder::build_forward_erc20_transaction(Address::from_str(HYPEREVM_USDC_TOKEN_ID)?, deposit_call, "0"))
}

fn build_swap_forward_transaction(route: &MayanMonoChainQuote, deposit_call: &EvmForwarderProtocolCall) -> Result<EvmTransaction, SwapperError> {
    let min_middle_amount = fractional_amount::<U256>(&route.min_amount_out, CCTP_TOKEN_DECIMALS)?;
    let swap = EvmSwapForwardData::new(
        route.evm_swap_router_address.as_deref().ok_or(SwapperError::InvalidRoute)?,
        route.evm_swap_router_calldata.as_deref().ok_or(SwapperError::InvalidRoute)?,
        HYPEREVM_USDC_TOKEN_ID,
        min_middle_amount,
    )?;

    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        return Ok(evm_builder::build_swap_and_forward_eth_transaction(
            deposit_call,
            swap,
            deposit_call.amount_in,
            deposit_call.amount_in.to_string(),
        ));
    }

    Ok(evm_builder::build_swap_and_forward_erc20_transaction(
        Address::from_str(&route.from_token.contract)?,
        deposit_call,
        swap,
        "0",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mayan::constants::MAYAN_FORWARDER;
    use crate::mayan::model::{MayanQuoteCommon, MayanToken};
    use primitives::{Chain, asset_constants::HYPERCORE_SPOT_USDC_TOKEN_ID};

    #[tokio::test]
    async fn test_build_direct_hyperevm_to_hypercore_transaction() {
        let mut quote = Quote::mock(Chain::Hyperliquid, Some(HYPEREVM_USDC_TOKEN_ID));
        quote.request.destination_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string();
        quote.from_value = "1000000".to_string();
        let route = MayanMonoChainQuote {
            common: MayanQuoteCommon {
                effective_amount_in64: "1000000".to_string(),
                min_amount_out: serde_json::json!(1),
                from_chain: WormholeChain::Hyperevm.name().to_string(),
                to_chain: WormholeChain::Hypercore.name().to_string(),
                from_token: MayanToken {
                    contract: HYPEREVM_USDC_TOKEN_ID.to_string(),
                    w_chain_id: 47,
                    decimals: 6,
                    verified_address: None,
                },
                to_token: MayanToken {
                    contract: crate::mayan::constants::HYPERCORE_SPOT_USDC_CONTRACT.to_string(),
                    w_chain_id: 65000,
                    decimals: 6,
                    verified_address: Some(HYPERCORE_SPOT_USDC_TOKEN_ID.to_string()),
                },
                referrer_bps: Some(50),
                ..Default::default()
            },
            mono_chain_mayan_contract: "0xd788230d2d3d5460b030cc4f21f17250276399d1".to_string(),
            evm_swap_router_address: None,
            evm_swap_router_calldata: None,
        };

        let transaction = build(&quote, &route).await.unwrap();

        assert_eq!(transaction.to, MAYAN_FORWARDER);
        assert_eq!(transaction.value, "0");
        assert!(transaction.data.starts_with("0x"));
        assert!(!transaction.data.is_empty());
    }
}
