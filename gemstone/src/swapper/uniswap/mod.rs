use crate::{
    asset,
    config::evm_chain,
    network::{AlienProvider, AlienTarget},
    swapper::{GemSwapperError, GemSwapperTrait},
};
use alloy_core::{
    primitives::{Bytes, Uint},
    sol_types::{abi::token, SolCall},
};
use async_trait::async_trait;
use gem_evm::uniswap::IQuoterV2;
use primitives::{AssetId, Chain, ChainType, EVMChain, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
use std::{fmt::Debug, str::FromStr, sync::Arc};

static UNISWAP: &str = "Uniswap";

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

    fn get_asset_address(asset: &AssetId, evm_chain: EVMChain) -> String {
        match &asset.token_id {
            Some(token_id) => token_id.to_string(),
            None => evm_chain.weth_contract().unwrap().to_string(),
        }
    }

    fn build_path(request: &SwapQuoteProtocolRequest, evm_chain: EVMChain) -> Result<Bytes, GemSwapperError> {
        let token_in = Self::get_asset_address(&request.from_asset, evm_chain);
        let token_out = Self::get_asset_address(&request.to_asset, evm_chain);
        let bytes_in = hex::decode(&token_in).map_err(|_| GemSwapperError::InvalidAddress)?;
        let bytes_out = hex::decode(&token_out).map_err(|_| GemSwapperError::InvalidAddress)?;

        let mut bytes: Vec<u8> = vec![];
        bytes.extend(bytes_in);
        bytes.extend(bytes_out);
        Ok(Bytes::from(bytes))
    }
}

#[async_trait]
impl GemSwapperTrait for UniswapV3 {
    async fn fetch_quote(&self, request: SwapQuoteProtocolRequest, rpc_provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, GemSwapperError> {
        if !self.support_chain(request.from_asset.chain) {
            return Err(GemSwapperError::NotSupportedChain);
        }

        let evm_chain = EVMChain::from_chain(request.from_asset.chain).ok_or(GemSwapperError::NotSupportedChain)?;

        if evm_chain.weth_contract().is_none() {
            return Err(GemSwapperError::NotSupportedChain);
        }

        let path = Self::build_path(&request, evm_chain)?;
        let quoter_v2 = IQuoterV2::quoteExactInputCall {
            path,
            amountIn: Uint::from_str(&request.amount).unwrap(),
        };
        let calldata = quoter_v2.abi_encode();

        // FIXME: add AlienEthereumRPC
        let target = AlienTarget {
            url: "".into(),
            method: "POST".into(),
            headers: None,
            body: Some(calldata),
        };
        let result = rpc_provider.request(target).await.map_err(|_| GemSwapperError::NetworkError)?;

        let quote = IQuoterV2::quoteExactInputCall::abi_decode_returns(&result, true)
            .map_err(|_| GemSwapperError::NetworkError)?
            .amountOut;

        // FIXME: add data
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
