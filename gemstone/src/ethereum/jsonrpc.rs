use alloy_primitives::{hex::decode as HexDecode, U256};
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    debug_println,
    network::{jsonrpc_client_with_chain, AlienProvider},
    swapper::SwapperError,
};

use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    multicall3::{self, IMulticall3},
    parse_u256,
};
use primitives::{Chain, EVMChain};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxReceiptLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxReceipt {
    pub status: String,
    pub logs: Vec<TxReceiptLog>,
}

pub async fn fetch_gas_price(provider: Arc<dyn AlienProvider>, chain: Chain) -> Result<U256, SwapperError> {
    let call = EthereumRpc::GasPrice;
    let client = jsonrpc_client_with_chain(provider, chain);
    let value: String = client.request(call).await?;

    parse_u256(&value).ok_or(SwapperError::InvalidAmount("invalid gas price".into()))
}

pub async fn estimate_gas(provider: Arc<dyn AlienProvider>, chain: Chain, tx: TransactionObject) -> Result<U256, SwapperError> {
    let call = EthereumRpc::EstimateGas(tx, BlockParameter::Latest);
    let client = jsonrpc_client_with_chain(provider, chain);
    let value: String = client.request(call).await?;
    parse_u256(&value).ok_or(SwapperError::InvalidAmount("invalid gas limit".into()))
}

pub async fn fetch_tx_receipt(provider: Arc<dyn AlienProvider>, chain: Chain, tx_hash: &str) -> Result<TxReceipt, SwapperError> {
    let call = EthereumRpc::GetTransactionReceipt(tx_hash.into());
    let client = jsonrpc_client_with_chain(provider, chain);
    let result: TxReceipt = client.request(call).await?;
    Ok(result)
}

pub async fn multicall3_call(
    provider: Arc<dyn AlienProvider>,
    chain: &Chain,
    calls: Vec<IMulticall3::Call3>,
) -> Result<Vec<IMulticall3::Result>, SwapperError> {
    for (_idx, _call) in calls.iter().enumerate() {
        debug_println!(
            "call {_idx}: target {:?}, calldata: {:?}, allowFailure: {:?}",
            _call.target,
            hex::encode(&_call.callData),
            _call.allowFailure
        );
    }
    let evm_chain = EVMChain::from_chain(*chain).ok_or(SwapperError::NotSupportedChain)?;
    let multicall_address = multicall3::deployment_by_chain(&evm_chain);
    let data = IMulticall3::aggregate3Call { calls }.abi_encode();
    let call = EthereumRpc::Call(TransactionObject::new_call(multicall_address, data), BlockParameter::Latest);

    let client = jsonrpc_client_with_chain(provider.clone(), *chain);
    let result: String = client.request(call).await?;
    let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError(e.to_string()))?;

    let decoded = IMulticall3::aggregate3Call::abi_decode_returns(&hex_data).map_err(|_| SwapperError::ABIError("failed to decode aggregate3Call".into()))?;

    Ok(decoded)
}
