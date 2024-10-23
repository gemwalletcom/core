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
    #[error("RPC error: {msg}")]
    NetworkError { msg: String },
    #[error("ABI error: {msg}")]
    ABIError { msg: String },
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
    use primitives::{AssetId, Chain, SwapMode, SwapQuoteProtocolRequest};
    use serde_json;
    #[test]
    fn test_encode_quote_request() {
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

    #[test]
    fn test_decode_quote_request() {
        let json = r#"{"fromAsset":{"chain":"ethereum","tokenId":null},"toAsset":{"chain":"ethereum","tokenId":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"},"walletAddress":"0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7","destinationAddress":"0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7","amount":"10000000000000000","mode":"exactin","includeData":false}"#;
        let request: SwapQuoteProtocolRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            request,
            SwapQuoteProtocolRequest {
                from_asset: AssetId::from(Chain::Ethereum, None),
                to_asset: AssetId::from(Chain::Ethereum, Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".into())),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                amount: "10000000000000000".into(),
                mode: SwapMode::ExactIn,
                include_data: false,
            }
        );
    }
}
