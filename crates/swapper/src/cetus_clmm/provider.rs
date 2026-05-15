use super::{
    client::CetusClmm,
    constants::CETUS_CLMM_PUBLISHED_AT,
    model::{FeeSide, PoolRoute},
    tx_builder,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData, fees::quote_value_after_reserve_by_chain,
};
use async_trait::async_trait;
use gem_sui::coin_type_matches;
use primitives::Chain;

#[async_trait]
impl Swapper for CetusClmm {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Sui)]
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_value = quote_value_after_reserve_by_chain(request)?;
        let from_asset = request.from_asset.asset_id();
        let to_asset = request.to_asset.asset_id();
        let amount = from_value.parse::<u64>()?;
        if amount == 0 {
            return Err(SwapperError::InputAmountError { min_amount: Some("1".into()) });
        }

        let from_coin_type = CetusClmm::coin_type(&from_asset);
        let to_coin_type = CetusClmm::coin_type(&to_asset);
        let fee_side = FeeSide::select(&from_coin_type, &to_coin_type);

        let referral_fee = CetusClmm::referral_fee();
        let input_fee_amount = tx_builder::referral_fee_amount(amount, referral_fee.bps)?;
        let swap_amount = match fee_side {
            FeeSide::Input => amount
                .checked_sub(input_fee_amount)
                .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus CLMM referral fee exceeds input amount".into()))?,
            FeeSide::Output => amount,
        };

        let slippage_bps = request.options.slippage.bps;
        let hops = self.find_route_hops(&from_coin_type, &to_coin_type, swap_amount).await?;
        let gross_amount_out = hops.last().map(|h| h.amount_out).unwrap_or_default();
        let fee_amount = match fee_side {
            FeeSide::Input => input_fee_amount,
            FeeSide::Output => tx_builder::referral_fee_amount(gross_amount_out, referral_fee.bps)?,
        };
        let route = PoolRoute { hops, fee_amount, fee_side };

        Ok(Quote {
            from_value,
            to_value: route.net_amount_out().to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset,
                    output: to_asset,
                    route_data: serde_json::to_string(&route)?,
                }],
                slippage_bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(0),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route_entry = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route: PoolRoute = serde_json::from_str(&route_entry.route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let request_from = CetusClmm::coin_type(&quote.request.from_asset.asset_id());
        let request_to = CetusClmm::coin_type(&quote.request.to_asset.asset_id());
        if !coin_type_matches(&request_from, route.input_coin_type()) || !coin_type_matches(&request_to, route.output_coin_type()) {
            return Err(SwapperError::InvalidRoute);
        }

        tx_builder::build_quote_data(&self.sui_client, quote, &route, &CetusClmm::referral_fee(), CETUS_CLMM_PUBLISHED_AT).await
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{AssetId, asset_constants::SUI_USDC_TOKEN_ID};
    use std::sync::Arc;

    const TEST_WALLET: &str = "0x9059c9d089cebc40fbe8c365782ab1285b99959fa386f5a5fc9cdf861a3e0b17";
    const BLUE_TOKEN_ID: &str = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";

    fn print_quote(label: &str, quote: &Quote) -> Result<PoolRoute, SwapperError> {
        let route_entry = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route: PoolRoute = serde_json::from_str(&route_entry.route_data)?;
        println!(
            "{label} quote: from_value={}, to_value={}, hops={}, gross_out={}, net_out={}, fee_side={:?}, fee_amount={}",
            quote.from_value,
            quote.to_value,
            route.hops.len(),
            route.gross_amount_out(),
            route.net_amount_out(),
            route.fee_side,
            route.fee_amount
        );
        for (index, hop) in route.hops.iter().enumerate() {
            println!(
                "{label} hop {}: amount_in={}, amount_out={}, a2b={}, pool_id={}",
                index + 1,
                hop.amount_in,
                hop.amount_out,
                hop.a2b,
                hop.pool_id
            );
        }
        Ok(route)
    }

    fn print_quote_data(label: &str, quote_data: &SwapperQuoteData) {
        println!(
            "{label} quote_data: to={}, value={}, data_len={}, gas_limit={:?}",
            quote_data.to,
            quote_data.value,
            quote_data.data.len(),
            quote_data.gas_limit
        );
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_fetch_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: "1500000000".to_string(),
            options: Options::new_with_slippage(50.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        print_quote("SUI->USDC", &quote)?;
        print_quote_data("SUI->USDC", &quote_data);

        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.gas_limit.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_fetch_quote_usdc_to_sui() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: "100000".to_string(),
            options: Options::new_with_slippage(50.into()),
        };

        let quote = provider.get_quote(&request).await?;
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        print_quote("USDC->SUI", &quote)?;
        print_quote_data("USDC->SUI", &quote_data);

        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.gas_limit.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_discovers_blue_sui_pool() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(BLUE_TOKEN_ID.to_string()))),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: "100000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        print_quote("SUI->BLUE", &quote)?;
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_routes_usdc_to_blue() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(BLUE_TOKEN_ID.to_string()))),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: "100000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        let route = print_quote("USDC->BLUE", &quote)?;
        assert!(!route.hops.is_empty() && route.hops.len() <= 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_clmm_provider_routes_blue_to_usdc() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = CetusClmm::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(BLUE_TOKEN_ID.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Sui, Some(SUI_USDC_TOKEN_ID.to_string()))),
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: "10000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        let route = print_quote("BLUE->USDC", &quote)?;
        assert!(!route.hops.is_empty() && route.hops.len() <= 2);
        Ok(())
    }
}
