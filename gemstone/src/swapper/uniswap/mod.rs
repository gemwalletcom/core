use crate::{
    network::{AlienProvider, JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
    swapper::{GemSwapperError, GemSwapperTrait},
};
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::{FeeTier, IQuoterV2},
};
use primitives::{AssetId, Chain, ChainType, EVMChain, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};

use alloy_core::{
    primitives::{hex::decode as HexDecode, Bytes, Uint},
    sol_types::SolCall,
};
use alloy_primitives::aliases::U24;

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

    fn build_path(request: &SwapQuoteProtocolRequest, evm_chain: EVMChain, fee_tier: FeeTier) -> Result<Bytes, GemSwapperError> {
        let token_in = Self::get_asset_address(&request.from_asset, evm_chain)?;
        let token_out = Self::get_asset_address(&request.to_asset, evm_chain)?;

        Self::build_path_with_token(token_in, token_out, fee_tier)
    }

    fn build_path_with_token(token_in: EthereumAddress, token_out: EthereumAddress, fee_tier: FeeTier) -> Result<Bytes, GemSwapperError> {
        let mut bytes: Vec<u8> = vec![];
        let fee = U24::from(fee_tier as u32);

        bytes.extend(&token_in.bytes);
        bytes.extend(&fee.to_be_bytes_vec());
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
        let path = Self::build_path(&request, evm_chain, FeeTier::Low)?;
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
        let decoded = HexDecode(&result).unwrap();
        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded, true)
            .map_err(|err| GemSwapperError::ABIError { msg: err.to_string() })?
            .amountOut;

        let swap_data: Option<SwapQuoteData> = None;
        if request.include_data {
            return Err(GemSwapperError::NotImplemented);
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_core::{hex::decode as HexDecode, hex::encode_prefixed as HexEncode};
    use alloy_primitives::aliases::U256;

    #[test]
    fn test_build_path() {
        // Optimism WETH
        let token0 = EthereumAddress::parse("0x4200000000000000000000000000000000000006").unwrap();
        // USDC
        let token1 = EthereumAddress::parse("0x0b2c639c533813f4aa9d7837caf62653d097ff85").unwrap();
        let bytes = UniswapV3::build_path_with_token(token0, token1, FeeTier::Low).unwrap();

        assert_eq!(
            HexEncode(bytes),
            "0x42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff85"
        )
    }

    #[test]
    fn test_decode_quoter_v2_response() {
        let result = "0x0000000000000000000000000000000000000000000000000000000001884eee000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000014b1e00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000004d04db53840b0aec247bb9bd3ffc00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001";
        let decoded = HexDecode(result).unwrap();
        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&decoded, false).unwrap();

        assert_eq!(quote.amountOut, U256::from(25710318));
        assert_eq!(quote.gasEstimate, U256::from(84766));
    }
}
