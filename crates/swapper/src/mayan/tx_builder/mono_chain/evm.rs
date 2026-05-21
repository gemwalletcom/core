use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    fees::default_referral_address,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        model::MayanMonoChainQuote,
        tx_builder::{
            amount::fractional_amount,
            evm::{self as evm_builder, EvmTransaction, MayanForwarder},
            hypercore::hypercore_deposit_dex,
            route::quote_destination_address,
        },
        wormhole_chain::WormholeChain,
    },
};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{SolCall, sol};
use gem_evm::EVM_ZERO_ADDRESS;
use primitives::{asset_constants::HYPEREVM_USDC_TOKEN_ID, decode_hex};
use std::{str::FromStr, sync::Arc};

const MAX_BPS: u32 = 10_000;

sol! {
    interface MayanHyperCoreDeposit {
        function depositToHyperCore(address tokenIn, uint256 amountIn, uint16 referrerBps, address referrerAddr, address destAddr, uint32 destDex) external;
    }
}

struct HyperCoreDepositCall {
    contract_address: Address,
    amount_in: U256,
    data: Vec<u8>,
}

impl HyperCoreDepositCall {
    fn new(quote: &Quote, route: &MayanMonoChainQuote) -> Result<Self, SwapperError> {
        if route.from_chain != WormholeChain::Hyperevm.name() || route.to_chain != WormholeChain::Hypercore.name() {
            return Err(SwapperError::InvalidRoute);
        }

        let contract_address = Address::from_str(&route.mono_chain_mayan_contract)?;
        let amount_in = U256::from_str(&quote.from_value)?;
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

        Ok(Self {
            contract_address,
            amount_in,
            data,
        })
    }
}

pub async fn build_quote_data(quote: &Quote, route: &MayanMonoChainQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError> {
    evm_builder::build_quote_data(build(quote, route), quote, rpc_provider).await
}

async fn build(quote: &Quote, route: &MayanMonoChainQuote) -> Result<EvmTransaction, SwapperError> {
    let deposit_call = HyperCoreDepositCall::new(quote, route)?;
    if route.from_token.contract.eq_ignore_ascii_case(HYPEREVM_USDC_TOKEN_ID) {
        return build_direct_forward_transaction(deposit_call);
    }

    build_swap_forward_transaction(route, deposit_call)
}

fn build_direct_forward_transaction(deposit_call: HyperCoreDepositCall) -> Result<EvmTransaction, SwapperError> {
    let data = MayanForwarder::forwardERC20Call {
        tokenIn: Address::from_str(HYPEREVM_USDC_TOKEN_ID)?,
        amountIn: deposit_call.amount_in,
        permitParams: MayanForwarder::PermitParams::default(),
        mayanProtocol: deposit_call.contract_address,
        protocolData: Bytes::from(deposit_call.data),
    }
    .abi_encode();
    Ok(EvmTransaction::forwarder("0", data))
}

fn build_swap_forward_transaction(route: &MayanMonoChainQuote, deposit_call: HyperCoreDepositCall) -> Result<EvmTransaction, SwapperError> {
    let swap_router_address = Address::from_str(route.evm_swap_router_address.as_deref().ok_or(SwapperError::InvalidRoute)?)?;
    let swap_router_calldata = Bytes::from(decode_hex(route.evm_swap_router_calldata.as_deref().ok_or(SwapperError::InvalidRoute)?)?);
    let middle_token = Address::from_str(HYPEREVM_USDC_TOKEN_ID)?;
    let min_middle_amount = fractional_amount::<U256>(&route.min_amount_out, CCTP_TOKEN_DECIMALS)?;

    if route.from_token.contract.eq_ignore_ascii_case(EVM_ZERO_ADDRESS) {
        let data = MayanForwarder::swapAndForwardEthCall {
            amountIn: deposit_call.amount_in,
            swapProtocol: swap_router_address,
            swapData: swap_router_calldata,
            middleToken: middle_token,
            minMiddleAmount: min_middle_amount,
            mayanProtocol: deposit_call.contract_address,
            mayanData: Bytes::from(deposit_call.data),
        }
        .abi_encode();
        return Ok(EvmTransaction::forwarder(deposit_call.amount_in.to_string(), data));
    }

    let data = MayanForwarder::swapAndForwardERC20Call {
        tokenIn: Address::from_str(&route.from_token.contract)?,
        amountIn: deposit_call.amount_in,
        permitParams: MayanForwarder::PermitParams::default(),
        swapProtocol: swap_router_address,
        swapData: swap_router_calldata,
        middleToken: middle_token,
        minMiddleAmount: min_middle_amount,
        mayanProtocol: deposit_call.contract_address,
        mayanData: Bytes::from(deposit_call.data),
    }
    .abi_encode();
    Ok(EvmTransaction::forwarder("0", data))
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
