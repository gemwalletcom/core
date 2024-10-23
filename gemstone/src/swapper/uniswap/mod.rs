use crate::{
    network::{AlienProvider, JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
    swapper::{GemSwapperError, GemSwapperTrait},
};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::IQuoterV2,
};
use primitives::{AssetId, Chain, ChainType, EVMChain, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};

use alloy_core::{
    primitives::{Bytes, Uint},
    sol_types::SolCall,
};
use async_trait::async_trait;
use std::{fmt::Debug, str::FromStr, sync::Arc};

mod deployment;

static UNISWAP: &str = "Uniswap";

impl From<&EthereumRpc> for JsonRpcRequest {
    fn from(val: &EthereumRpc) -> Self {
        match val {
            EthereumRpc::GasPrice => JsonRpcRequest {
                method: val.method_name().into(),
                params: None,
                id: 1,
            },
            EthereumRpc::GetBalance(address) => {
                let params: Vec<String> = vec![address.to_string()];
                JsonRpcRequest {
                    method: val.method_name().into(),
                    params: Some(serde_json::to_string(&params).unwrap()),
                    id: 1,
                }
            }
            EthereumRpc::Call(transaction, _block) => {
                let params = vec![transaction];
                JsonRpcRequest {
                    method: val.method_name().into(),
                    params: Some(serde_json::to_string(&params).unwrap()),
                    id: 1,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct UniswapV3 {}

impl UniswapV3 {
    pub fn new() -> Self {
        Self {}
    }

    pub fn support_chain(&self, chain: Chain) -> bool {
        matches!(
            chain,
            Chain::Ethereum | Chain::Polygon | Chain::AvalancheC | Chain::Arbitrum | Chain::Optimism | Chain::Base | Chain::SmartChain
        )
    }

    fn get_asset_address(asset: &AssetId, evm_chain: EVMChain) -> Result<EthereumAddress, GemSwapperError> {
        let str = match &asset.token_id {
            Some(token_id) => token_id.to_string(),
            None => evm_chain.weth_contract().unwrap().to_string(),
        };
        let address = EthereumAddress::parse(&str);
        match address {
            Some(address) => Ok(address),
            None => Err(GemSwapperError::InvalidAddress),
        }
    }

    fn build_path(request: &SwapQuoteProtocolRequest, evm_chain: EVMChain) -> Result<Bytes, GemSwapperError> {
        let token_in = Self::get_asset_address(&request.from_asset, evm_chain)?;
        let token_out = Self::get_asset_address(&request.to_asset, evm_chain)?;

        let mut bytes: Vec<u8> = vec![];
        bytes.extend(&token_in.bytes);
        bytes.extend(&token_out.bytes);
        Ok(Bytes::from(bytes))
    }

    async fn jsonrpc_call(&self, request: EthereumRpc, provider: Arc<dyn AlienProvider>, chain: Chain) -> Result<JsonRpcResponse, GemSwapperError> {
        let req = JsonRpcRequest::from(&request);
        let result = provider.jsonrpc_call(vec![req], chain).await;
        match result {
            Ok(results) => {
                let result = results.first().ok_or(GemSwapperError::NetworkError { msg: "No result".into() })?;
                match result {
                    JsonRpcResult::Value(value) => Ok(value.clone()),
                    JsonRpcResult::Error(err) => Err(GemSwapperError::NetworkError { msg: err.message.clone() }),
                }
            }
            Err(err) => Err(GemSwapperError::NetworkError { msg: err.to_string() }),
        }
    }
}

#[async_trait]
impl GemSwapperTrait for UniswapV3 {
    async fn fetch_quote(&self, request: SwapQuoteProtocolRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, GemSwapperError> {
        // Prevent swaps on unsupported chains
        if !self.support_chain(request.from_asset.chain) {
            return Err(GemSwapperError::NotSupportedChain);
        }

        let evm_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(GemSwapperError::NotSupportedChain)?;
        let deployment = deployment::get_deployment_by_chain(request.from_asset.chain).ok_or(GemSwapperError::NotSupportedChain)?;

        evm_chain.weth_contract().ok_or(GemSwapperError::NotSupportedChain)?;

        // Build path for QuoterV2
        let path = Self::build_path(&request, evm_chain)?;
        let quoter_v2 = IQuoterV2::quoteExactInputCall {
            path,
            amountIn: Uint::from_str(&request.amount).unwrap(),
        };

        let calldata = quoter_v2.abi_encode();
        let eth_call = EthereumRpc::Call(
            TransactionObject::new_call(&request.wallet_address, deployment.quoter_v2, calldata),
            BlockParameter::Latest,
        );
        let response = self.jsonrpc_call(eth_call, provider, request.from_asset.chain).await?;
        let result = response.result.ok_or(GemSwapperError::NetworkError { msg: "No result".into() })?;

        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&result, true)
            .map_err(|err| GemSwapperError::ABIError { msg: err.to_string() })?
            .amountOut;

        // FIXME: encode swap data
        let swap_data: Option<SwapQuoteData> = None;
        if request.include_data {
            todo!("call universal router to encode data");
        }

        Ok(SwapQuote {
            chain_type: ChainType::Ethereum,
            from_amount: request.amount.clone(),
            to_amount: quote.to_string(),
            fee_percent: 0.0,
            provider: UNISWAP.into(),
            data: swap_data,
            approval: None,
        })
    }
}
