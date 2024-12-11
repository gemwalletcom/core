use crate::{
    network::{jsonrpc::batch_jsonrpc_call, AlienProvider, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResponse, JsonRpcResult},
    swapper::{
        approval::{check_approval, CheckApprovalType},
        models::*,
        slippage::apply_slippage_in_bp,
        weth_address, GemSwapProvider, SwapperError,
    },
};
use gem_evm::{
    across::deployment::{self, AcrossDeployment},
    address::EthereumAddress,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use primitives::{AssetId, Chain, EVMChain};

use alloy_core::{
    primitives::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        Address, Bytes, U256,
    },
    sol_types::SolCall,
};
use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug, Default)]
pub struct Across {}
impl Across {
    pub fn is_supported_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        let from = weth_address::normalize_asset(from_asset).unwrap();
        let to = weth_address::normalize_asset(to_asset).unwrap();

        let asset_mappings = AcrossDeployment::asset_mappings();
        for set in asset_mappings.iter() {
            if set.contains(&from) && set.contains(&to) {
                return true;
            }
        }
        false
    }
}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_chains(&self) -> Vec<Chain> {
        AcrossDeployment::deployed_chains()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // does not support same chain swap
        if request.from_asset.chain == request.to_asset.chain {
            return Err(SwapperError::NotSupportedPair);
        }
        let deployment = AcrossDeployment::deployment_by_chain(&request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_pair(&request.from_asset, &request.to_asset) {
            return Err(SwapperError::NotSupportedPair);
        }

        Err(SwapperError::NotImplemented)
    }
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        Err(SwapperError::NotImplemented)
    }
    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::constants::*;

    #[test]
    fn test_is_supported_pair() {
        let weth_eth: AssetId = WETH_ETH.into();
        let weth_op: AssetId = WETH_OP.into();
        let usdc_eth: AssetId = USDC_ETH.into();
        let usdc_arb: AssetId = USDC_ARB.into();

        assert!(Across::is_supported_pair(&weth_eth, &weth_op));
        assert!(Across::is_supported_pair(&usdc_eth, &usdc_arb));
        assert!(!Across::is_supported_pair(&weth_eth, &usdc_eth));
    }
}
