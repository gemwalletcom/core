use crate::network::AlienProvider;
use async_trait::async_trait;
use primitives::{SwapQuote, SwapQuoteProtocolRequest};
use std::{fmt::Debug, sync::Arc};
mod custom_types;
mod uniswap;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GemSwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
    #[error("Invalid address")]
    InvalidAddress,
    #[error("RPC error: {message}")]
    NetworkError { message: String },
    #[error("ABI error: {message}")]
    ABIError { message: String },
    #[error("No quote available")]
    NoQuoteAvailable,
}

#[async_trait]
pub trait GemSwapperTrait: Send + Sync + Debug {
    async fn fetch_quote(&self, request: SwapQuoteProtocolRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, GemSwapperError>;
}

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    pub rpc_provider: Arc<dyn AlienProvider>,
    pub swappers: Vec<Box<dyn GemSwapperTrait>>,
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            rpc_provider,
            swappers: vec![Box::new(uniswap::UniswapV3::new())],
        }
    }

    async fn fetch_quote(&self, request: SwapQuoteProtocolRequest) -> Result<SwapQuote, GemSwapperError> {
        for swapper in self.swappers.iter() {
            let quote = swapper.fetch_quote(request.clone(), self.rpc_provider.clone()).await;
            match quote {
                Ok(quote) => return Ok(quote),
                Err(err) => {
                    println!("error swapping: {}, {:?}", err, err);
                }
            }
        }
        Err(GemSwapperError::NoQuoteAvailable)
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, SwapQuoteProtocolRequest};
    use serde_json;
    #[test]
    fn test_swap_provider() {
        let request = SwapQuoteProtocolRequest {
            from_asset: AssetId::from(Chain::Ethereum, None),
            to_asset: AssetId::from(Chain::Ethereum, None),
            wallet_address: String::from("0x1234567890abcdef"),
            destination_address: String::from("0x1234567890abcdef"),
            amount: String::from("0.0"),
            mode: primitives::SwapMode::ExactIn,
            include_data: false,
        };

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            json,
            r#"{"fromAsset":{"chain":"ethereum","tokenId":null},"toAsset":{"chain":"ethereum","tokenId":null},"walletAddress":"0x1234567890abcdef","destinationAddress":"0x1234567890abcdef","amount":"0.0","mode":"exactin","includeData":false}"#
        );
    }
}
