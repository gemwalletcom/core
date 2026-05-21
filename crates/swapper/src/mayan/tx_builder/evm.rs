use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, check_approval_erc20, get_swap_gas_limit_with_approval},
    mayan::constants::MAYAN_FORWARDER,
};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{SolCall, sol};
use futures::try_join;
use primitives::{AssetId, ChainType, decode_hex, hex, swap::ApprovalData};
use std::{future::Future, str::FromStr, sync::Arc};

sol! {
    interface MayanForwarder {
        #[derive(Default)]
        struct PermitParams {
            uint256 value;
            uint256 deadline;
            uint8 v;
            bytes32 r;
            bytes32 s;
        }

        function forwardERC20(address tokenIn, uint256 amountIn, PermitParams permitParams, address mayanProtocol, bytes protocolData) external payable;
        function swapAndForwardERC20(
            address tokenIn,
            uint256 amountIn,
            PermitParams permitParams,
            address swapProtocol,
            bytes swapData,
            address middleToken,
            uint256 minMiddleAmount,
            address mayanProtocol,
            bytes mayanData
        ) external payable;
        function swapAndForwardEth(
            uint256 amountIn,
            address swapProtocol,
            bytes swapData,
            address middleToken,
            uint256 minMiddleAmount,
            address mayanProtocol,
            bytes mayanData
        ) external payable;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::mayan::tx_builder) struct EvmTransaction {
    pub(in crate::mayan::tx_builder) to: String,
    pub(in crate::mayan::tx_builder) value: String,
    pub(in crate::mayan::tx_builder) data: String,
}

impl EvmTransaction {
    pub(in crate::mayan::tx_builder) fn forwarder(value: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            to: MAYAN_FORWARDER.to_string(),
            value: value.into(),
            data: hex::encode_with_0x(&data),
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::mayan::tx_builder) struct EvmForwarderProtocolCall {
    pub amount_in: U256,
    pub protocol_address: Address,
    pub data: Vec<u8>,
}

impl EvmForwarderProtocolCall {
    pub(in crate::mayan::tx_builder) fn new(amount_in: U256, protocol_address: Address, data: Vec<u8>) -> Self {
        Self {
            amount_in,
            protocol_address,
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::mayan::tx_builder) struct EvmSwapForwardData {
    swap_router_address: Address,
    swap_router_calldata: Bytes,
    middle_token: Address,
    min_middle_amount: U256,
}

impl EvmSwapForwardData {
    pub(in crate::mayan::tx_builder) fn new(swap_router_address: &str, swap_router_calldata: &str, middle_token: &str, min_middle_amount: U256) -> Result<Self, SwapperError> {
        Ok(Self {
            swap_router_address: Address::from_str(swap_router_address)?,
            swap_router_calldata: Bytes::from(decode_hex(swap_router_calldata)?),
            middle_token: Address::from_str(middle_token)?,
            min_middle_amount,
        })
    }
}

pub(in crate::mayan::tx_builder) fn build_forward_erc20_transaction(token_in: Address, protocol_call: &EvmForwarderProtocolCall, value: impl Into<String>) -> EvmTransaction {
    let data = MayanForwarder::forwardERC20Call {
        tokenIn: token_in,
        amountIn: protocol_call.amount_in,
        permitParams: MayanForwarder::PermitParams::default(),
        mayanProtocol: protocol_call.protocol_address,
        protocolData: Bytes::from(protocol_call.data.clone()),
    }
    .abi_encode();
    EvmTransaction::forwarder(value, data)
}

pub(in crate::mayan::tx_builder) fn build_swap_and_forward_eth_transaction(
    protocol_call: &EvmForwarderProtocolCall,
    swap: EvmSwapForwardData,
    amount_in: U256,
    value: impl Into<String>,
) -> EvmTransaction {
    let data = MayanForwarder::swapAndForwardEthCall {
        amountIn: amount_in,
        swapProtocol: swap.swap_router_address,
        swapData: swap.swap_router_calldata,
        middleToken: swap.middle_token,
        minMiddleAmount: swap.min_middle_amount,
        mayanProtocol: protocol_call.protocol_address,
        mayanData: Bytes::from(protocol_call.data.clone()),
    }
    .abi_encode();
    EvmTransaction::forwarder(value, data)
}

pub(in crate::mayan::tx_builder) fn build_swap_and_forward_erc20_transaction(
    token_in: Address,
    protocol_call: &EvmForwarderProtocolCall,
    swap: EvmSwapForwardData,
    value: impl Into<String>,
) -> EvmTransaction {
    let data = MayanForwarder::swapAndForwardERC20Call {
        tokenIn: token_in,
        amountIn: protocol_call.amount_in,
        permitParams: MayanForwarder::PermitParams::default(),
        swapProtocol: swap.swap_router_address,
        swapData: swap.swap_router_calldata,
        middleToken: swap.middle_token,
        minMiddleAmount: swap.min_middle_amount,
        mayanProtocol: protocol_call.protocol_address,
        mayanData: Bytes::from(protocol_call.data.clone()),
    }
    .abi_encode();
    EvmTransaction::forwarder(value, data)
}

pub(in crate::mayan::tx_builder) async fn build_quote_data(
    transaction: impl Future<Output = Result<EvmTransaction, SwapperError>>,
    quote: &Quote,
    rpc_provider: Arc<dyn RpcProvider>,
) -> Result<SwapperQuoteData, SwapperError> {
    let approval = approval_data(
        quote.request.wallet_address.clone(),
        quote.request.from_asset.asset_id(),
        MAYAN_FORWARDER,
        U256::from_str(&quote.from_value)?,
        rpc_provider,
    );
    let (transaction, approval) = try_join!(transaction, approval)?;
    let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_EVM_SWAP_GAS_LIMIT);
    Ok(SwapperQuoteData::new_contract(transaction.to, transaction.value, transaction.data, approval, gas_limit))
}

async fn approval_data(wallet_address: String, asset: AssetId, spender: &str, amount: U256, rpc_provider: Arc<dyn RpcProvider>) -> Result<Option<ApprovalData>, SwapperError> {
    if asset.is_native() || asset.chain.chain_type() != ChainType::Ethereum {
        return Ok(None);
    }
    let token = asset.token_id.ok_or(SwapperError::NotSupportedAsset)?;
    Ok(check_approval_erc20(wallet_address, token, spender.to_string(), amount, rpc_provider, &asset.chain)
        .await?
        .approval_data())
}
