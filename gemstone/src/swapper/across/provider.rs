use crate::swapper::across::asset::AcrossChainAsset;
use crate::swapper::across::chain::AcrossChainName;
use crate::swapper::across::client::AcrossSwapClient;
use crate::{
    network::AlienProvider,
    swapper::{models::*, GemSwapProvider, SwapperError},
};
use async_trait::async_trait;
use gem_evm::uniswap::FeeTier;
use num_bigint::BigInt;
use primitives::Chain;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Across {
    pub chain: Chain,
}

impl Default for Across {
    fn default() -> Self {
        Self { chain: Chain::Solana }
    }
}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_chains(&self) -> Vec<Chain> {
        AcrossChainName::all().iter().map(|name| name.chain()).collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // Prevent swaps on unsupported chains
        if !self.supported_chains().contains(&request.from_asset.chain) {
            return Err(SwapperError::NotSupportedChain);
        }
        let client = AcrossSwapClient::new(provider);
        let from_asset = AcrossChainAsset::from_asset_id(request.clone().from_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = AcrossChainAsset::from_asset_id(request.clone().to_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let quote = client
            .get_quote(&self.get_endpoint(), from_asset.clone(), to_asset.clone(), request.clone().value)
            .await?;

        let to_value = self.value_to(request.value.to_string(), to_asset.decimals as i32);

        let quote = SwapQuote {
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    gas_estimate: None,
                    route_data: "".to_string(),
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        // TODO: the transaction status from the RPC
        Ok(true)
    }
}

impl Across {
    pub fn get_endpoint(&self) -> String {
        "https://app.across.to".into()
    }
    fn value_from(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow(decimals as u32)
        } else {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow(decimals.unsigned_abs())
        }
    }

    fn value_to(&self, value: String,  decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow((decimals).unsigned_abs())
        } else {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow((decimals).unsigned_abs())
        }
    }
}

#[cfg(test)]
mod tests {
}
