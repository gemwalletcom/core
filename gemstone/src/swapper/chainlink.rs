use alloy_core::sol_types::SolCall;
use alloy_primitives::hex::decode as HexDecode;
use num_bigint::BigInt;
use num_traits::FromBytes;
use std::sync::Arc;

use crate::{
    network::{
        jsonrpc::{jsonrpc_call, JsonRpcResult},
        AlienProvider,
    },
    swapper::SwapperError,
};
use gem_evm::{
    chainlink::contract::{AggregatorInterface, CHAINLINK_ETH_USD_FEED},
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    multicall3::{create_call3, decode_call3_return, IMulticall3},
};
use primitives::Chain;

pub struct ChainLinkPriceFeed {
    pub contract: String,
    pub provider: Arc<dyn AlienProvider>,
    pub chain: Chain,
}

impl ChainLinkPriceFeed {
    pub fn new_eth_usd_feed(provider: Arc<dyn AlienProvider>) -> ChainLinkPriceFeed {
        ChainLinkPriceFeed {
            contract: CHAINLINK_ETH_USD_FEED.into(),
            provider,
            chain: Chain::Ethereum,
        }
    }

    pub fn latest_round_call3(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, AggregatorInterface::latestRoundDataCall {})
    }

    // Price is in 8 decimals
    pub fn decoded_answer(result: &IMulticall3::Result) -> Result<BigInt, SwapperError> {
        let answer = decode_call3_return::<AggregatorInterface::latestRoundDataCall>(result)
            .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            .answer;
        let price = BigInt::from_le_bytes(&answer.to_le_bytes::<32>());
        Ok(price)
    }

    #[allow(unused)]
    pub async fn fetch_latest_round(&self) -> Result<BigInt, SwapperError> {
        let data = AggregatorInterface::latestRoundDataCall {}.abi_encode();
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, self.provider.clone(), &self.chain).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        let answer = AggregatorInterface::latestRoundDataCall::abi_decode_returns(&hex_data, true)
            .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            .answer;

        Ok(BigInt::from_le_bytes(&answer.to_le_bytes::<32>()))
    }
}
