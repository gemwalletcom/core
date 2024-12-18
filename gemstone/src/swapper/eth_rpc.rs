use super::SwapperError;
use crate::network::{jsonrpc_call, AlienProvider, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResult};
use gem_evm::jsonrpc::{BlockParameter, EthereumRpc, TransactionObject};
use primitives::Chain;

use alloy_primitives::U256;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

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

impl JsonRpcRequestConvert for EthereumRpc {
    fn to_req(&self, id: u64) -> JsonRpcRequest {
        let method = self.method_name();
        let params: Vec<Value> = match self {
            EthereumRpc::GasPrice => vec![],
            EthereumRpc::GetBalance(address) => {
                vec![Value::String(address.to_string())]
            }
            EthereumRpc::Call(tx, block) => {
                let value = serde_json::to_value(tx).unwrap();
                vec![value, block.into()]
            }
            EthereumRpc::GetTransactionReceipt(tx_hash) => {
                vec![Value::String(tx_hash.to_string())]
            }
            EthereumRpc::EstimateGas(tx, block) => {
                let value = serde_json::to_value(tx).unwrap();
                vec![value, block.into()]
            }
        };

        JsonRpcRequest::new(id, method, params)
    }
}

pub async fn fetch_gas_price(provider: Arc<dyn AlienProvider>, chain: &Chain) -> Result<U256, SwapperError> {
    let call = EthereumRpc::GasPrice;
    let resp: JsonRpcResult<String> = jsonrpc_call(&call, provider.clone(), chain).await?;
    let gas_price = U256::from_str_radix(&resp.take()?, 16).map_err(|_| SwapperError::InvalidAmount)?;

    Ok(gas_price)
}

pub async fn estimate_gas(provider: Arc<dyn AlienProvider>, chain: &Chain, tx: TransactionObject) -> Result<U256, SwapperError> {
    let call = EthereumRpc::EstimateGas(tx, BlockParameter::Latest);
    let resp: JsonRpcResult<String> = jsonrpc_call(&call, provider.clone(), chain).await?;
    let limit = U256::from_str_radix(&resp.take()?, 16).map_err(|_| SwapperError::InvalidAmount)?;
    Ok(limit)
}

pub async fn fetch_tx_receipt(provider: Arc<dyn AlienProvider>, chain: &Chain, tx_hash: &str) -> Result<TxReceipt, SwapperError> {
    let call = EthereumRpc::GetTransactionReceipt(tx_hash.into());
    let resp: JsonRpcResult<TxReceipt> = jsonrpc_call(&call, provider.clone(), chain).await?;
    Ok(resp.take()?)
}
