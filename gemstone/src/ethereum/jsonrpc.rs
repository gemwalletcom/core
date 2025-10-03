use alloy_primitives::{U256, hex::decode as HexDecode};
use alloy_sol_types::SolCall;
use gem_client::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    debug_println,
    network::JsonRpcClient,
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

pub async fn fetch_gas_price<C>(client: &JsonRpcClient<C>) -> Result<U256, SwapperError>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
{
    let call = EthereumRpc::GasPrice;
    let value: String = client.request(call).await?;

    parse_u256(&value).ok_or(SwapperError::InvalidAmount("invalid gas price".into()))
}

pub async fn estimate_gas<C>(client: &JsonRpcClient<C>, tx: TransactionObject) -> Result<U256, SwapperError>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
{
    let call = EthereumRpc::EstimateGas(tx, BlockParameter::Latest);
    let value: String = client.request(call).await?;
    parse_u256(&value).ok_or(SwapperError::InvalidAmount("invalid gas limit".into()))
}

pub async fn fetch_tx_receipt<C>(client: &JsonRpcClient<C>, tx_hash: &str) -> Result<TxReceipt, SwapperError>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
{
    let call = EthereumRpc::GetTransactionReceipt(tx_hash.into());
    let result: TxReceipt = client.request(call).await?;
    Ok(result)
}

pub async fn multicall3_call<C>(
    client: &JsonRpcClient<C>,
    chain: &Chain,
    calls: Vec<IMulticall3::Call3>,
) -> Result<Vec<IMulticall3::Result>, SwapperError>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
{
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

    let result: String = client.request(call).await?;
    let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError(e.to_string()))?;

    let decoded = IMulticall3::aggregate3Call::abi_decode_returns(&hex_data).map_err(|_| SwapperError::ABIError("failed to decode aggregate3Call".into()))?;

    Ok(decoded)
}
