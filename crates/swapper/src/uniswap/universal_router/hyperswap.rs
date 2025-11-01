use crate::{ProviderType, SwapperProvider, uniswap::v3::UniversalRouterProvider};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{V3Deployment, get_hyperswap_deployment_by_chain},
};
use primitives::Chain;

#[derive(Debug)]
pub struct HyperswapUniversalRouter {
    pub provider: ProviderType,
}

impl Default for HyperswapUniversalRouter {
    fn default() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperswap),
        }
    }
}

impl UniversalRouterProvider for HyperswapUniversalRouter {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_hyperswap_deployment_by_chain(chain)
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use crate::{FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError, SwapperMode, SwapperProvider, uniswap};
    use primitives::{
        AssetId,
        asset_constants::{USDC_HYPEREVM_ASSET_ID, USDT_HYPEREVM_ASSET_ID},
    };
    use std::{sync::Arc, time::SystemTime};

    #[tokio::test]
    async fn test_hyperswap_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_hyperswap(network_provider.clone());

        let options = Options {
            slippage: 100.into(),
            fee: None,
            preferred_providers: vec![SwapperProvider::Hyperswap],
            use_max_amount: false,
        };

        let from_asset: AssetId = USDC_HYPEREVM_ASSET_ID.into();
        let to_asset: AssetId = USDT_HYPEREVM_ASSET_ID.into();

        let request = QuoteRequest {
            from_asset: from_asset.into(),
            to_asset: to_asset.into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "1000000".into(), // 1 USDC
            mode: SwapperMode::ExactIn,
            options,
        };

        let now = SystemTime::now();
        let quote = swap_provider.fetch_quote(&request).await?;
        let elapsed = SystemTime::now().duration_since(now).unwrap();

        println!("<== elapsed: {:?}", elapsed);
        println!("<== quote: {:?}", quote);
        let to_value = quote.to_value.parse::<u128>().unwrap();
        assert!(to_value > 0);

        let quote_data = swap_provider.fetch_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
        println!("<== quote_data: {:?}", quote_data);
        assert!(!quote_data.data.is_empty());

        Ok(())
    }
}
