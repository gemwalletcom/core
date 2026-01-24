use std::{fmt, marker::PhantomData};

use alloy_primitives::{Address, hex};
use alloy_sol_types::{SolCall, sol};
use gem_client::Client;
use primitives::chain_config::ChainStack;
use serde_json::json;

use crate::rpc::EthereumClient;

sol! {
    #[derive(Debug)]
    interface IMulticall3 {
        struct Call3 {
          address target;
          bool allowFailure;
          bytes callData;
        }

        struct Result {
          bool success;
          bytes returnData;
        }

        function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);
        function getCurrentBlockTimestamp() external view returns (uint256 timestamp);
    }
}

/// Handle returned when adding a call to the batch. Used to decode the result.
pub struct CallHandle<T> {
    index: usize,
    _marker: PhantomData<T>,
}

/// Results from executing a multicall batch
pub struct Multicall3Results {
    results: Vec<IMulticall3::Result>,
}

impl Multicall3Results {
    /// Decode the result for a specific call handle
    pub fn decode<T: SolCall>(&self, handle: &CallHandle<T::Return>) -> Result<T::Return, Multicall3Error> {
        let result = self.results.get(handle.index).ok_or_else(|| Multicall3Error(format!("invalid index: {}", handle.index)))?;

        if !result.success {
            return Err(Multicall3Error(format!("{} failed", T::SIGNATURE)));
        }

        T::abi_decode_returns(&result.returnData).map_err(|e| Multicall3Error(format!("{}: {:?}", T::SIGNATURE, e)))
    }
}

/// Builder for constructing multicall3 batches
pub struct Multicall3Builder<'a, C: Client + Clone> {
    client: &'a EthereumClient<C>,
    calls: Vec<IMulticall3::Call3>,
    block: Option<u64>,
}

impl<'a, C: Client + Clone> Multicall3Builder<'a, C> {
    pub fn new(client: &'a EthereumClient<C>) -> Self {
        Self {
            client,
            calls: Vec::new(),
            block: None,
        }
    }

    /// Add a contract call to the batch
    pub fn add<T: SolCall>(&mut self, target: Address, call: T) -> CallHandle<T::Return> {
        let index = self.calls.len();
        self.calls.push(IMulticall3::Call3 {
            target,
            allowFailure: true,
            callData: call.abi_encode().into(),
        });
        CallHandle { index, _marker: PhantomData }
    }

    /// Set the block number to execute at (default: latest)
    pub fn at_block(mut self, block: u64) -> Self {
        self.block = Some(block);
        self
    }

    /// Execute all calls in a single RPC request
    pub async fn execute(self) -> Result<Multicall3Results, Multicall3Error> {
        if self.calls.is_empty() {
            return Ok(Multicall3Results { results: vec![] });
        }

        let multicall_address = deployment_by_chain_stack(self.client.chain.chain_stack());
        let multicall_data = IMulticall3::aggregate3Call { calls: self.calls }.abi_encode();

        let block_param = self.block.map(|n| serde_json::Value::String(format!("0x{n:x}"))).unwrap_or_else(|| json!("latest"));

        let result: String = self
            .client
            .client
            .call(
                "eth_call",
                json!([{
                    "to": multicall_address,
                    "data": hex::encode_prefixed(&multicall_data)
                }, block_param]),
            )
            .await
            .map_err(|e| Multicall3Error(e.to_string()))?;

        let result_data = hex::decode(&result).map_err(|e| Multicall3Error(e.to_string()))?;

        let results = IMulticall3::aggregate3Call::abi_decode_returns(&result_data).map_err(|e| Multicall3Error(e.to_string()))?;

        Ok(Multicall3Results { results })
    }
}

#[derive(Debug)]
pub struct Multicall3Error(pub String);

impl fmt::Display for Multicall3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Multicall3Error {}

pub fn deployment_by_chain_stack(stack: ChainStack) -> &'static str {
    match stack {
        ChainStack::ZkSync => "0xF9cda624FBC7e059355ce98a31693d299FACd963",
        _ => "0xcA11bde05977b3631167028862bE2a173976CA11",
    }
}

// Helpers for direct Call3 creation (used by swapper crate)
pub fn create_call3(target: &str, call: impl SolCall) -> IMulticall3::Call3 {
    IMulticall3::Call3 {
        target: target.parse().unwrap(),
        allowFailure: true,
        callData: call.abi_encode().into(),
    }
}

pub fn decode_call3_return<T: SolCall>(result: &IMulticall3::Result) -> Result<T::Return, String> {
    if result.success {
        T::abi_decode_returns(&result.returnData).map_err(|e| format!("{}: {:?}", T::SIGNATURE, e))
    } else {
        Err(format!("{} failed", T::SIGNATURE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::IERC20;
    use alloy_primitives::U256;

    #[test]
    fn test_multicall3_results_decode_success() {
        let value = U256::from(42u64);
        let handle = CallHandle { index: 0, _marker: PhantomData };
        let results = Multicall3Results {
            results: vec![IMulticall3::Result {
                success: true,
                returnData: value.to_be_bytes::<32>().to_vec().into(),
            }],
        };

        let decoded = results.decode::<IERC20::balanceOfCall>(&handle).expect("decode should succeed");
        assert_eq!(decoded, value);
    }
}
