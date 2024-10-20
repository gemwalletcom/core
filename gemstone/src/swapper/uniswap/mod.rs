use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::{GemSwapperError, GemSwapperTrait},
};
use alloy_core::primitives::Uint;
use alloy_core::sol_types::SolCall;
use async_trait::async_trait;
use gem_evm::uniswap::IQuoterV2;
use primitives::{Chain, ChainType, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
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
}

#[async_trait]
impl GemSwapperTrait for UniswapV3 {
    async fn fetch_quote(&self, request: SwapQuoteProtocolRequest, rpc_provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, GemSwapperError> {
        if !self.support_chain(request.from_asset.chain) {
            return Err(GemSwapperError::NotSupportedChain);
        }

        // FIXME: concat path, handle weth
        let quoter_v2 = IQuoterV2::quoteExactInputCall {
            path: vec![].into(),
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
