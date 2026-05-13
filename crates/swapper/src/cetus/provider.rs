use super::{client::CetusClient, constants::DEFAULT_AGGREGATOR_PATH, model::RouterData, tx_builder};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    client_factory::create_client_with_chain,
    config::get_swap_proxy_url,
    fees::{ReferralFee, apply_slippage_in_bp, default_referral_fees, quote_value_after_reserve_by_chain},
};
use async_trait::async_trait;
use gem_client::DebugClientBounds;
use gem_sui::{SUI_COIN_TYPE, SuiClient, full_coin_type};
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

pub struct Cetus<C, R>
where
    C: DebugClientBounds,
    R: DebugClientBounds,
{
    provider: ProviderType,
    cetus_client: CetusClient<C>,
    sui_client: SuiClient<R>,
}

impl<C, R> Debug for Cetus<C, R>
where
    C: DebugClientBounds,
    R: DebugClientBounds,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cetus").field("provider", &self.provider).finish()
    }
}

impl Cetus<RpcClient, RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let cetus_client = RpcClient::new(get_swap_proxy_url(DEFAULT_AGGREGATOR_PATH), rpc_provider.clone());
        let sui_client = create_client_with_chain(rpc_provider, Chain::Sui);
        Self::with_clients(CetusClient::new(cetus_client), SuiClient::new(sui_client))
    }
}

impl<C, R> Cetus<C, R>
where
    C: DebugClientBounds,
    R: DebugClientBounds,
{
    pub fn with_clients(cetus_client: CetusClient<C>, sui_client: SuiClient<R>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::CetusAggregator),
            cetus_client,
            sui_client,
        }
    }

    fn referral_fee(request: &QuoteRequest) -> ReferralFee {
        request.options.fee.clone().map(|fees| fees.sui).unwrap_or_else(|| default_referral_fees().sui)
    }
}

#[async_trait]
impl<C, R> Swapper for Cetus<C, R>
where
    C: DebugClientBounds,
    R: DebugClientBounds,
{
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
        if from_asset.chain != Chain::Sui || to_asset.chain != Chain::Sui {
            return Err(SwapperError::NotSupportedChain);
        }
        let from = full_coin_type(from_asset.token_id.as_deref().unwrap_or(SUI_COIN_TYPE));
        let target = full_coin_type(to_asset.token_id.as_deref().unwrap_or(SUI_COIN_TYPE));
        let route = self.cetus_client.get_router(from, target, from_value.clone()).await?;
        let referral_fee = Self::referral_fee(request);
        let output_value = apply_slippage_in_bp(&route.amount_out, referral_fee.bps).to_string();

        Ok(Quote {
            from_value,
            to_value: output_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset,
                    output: to_asset,
                    route_data: serde_json::to_string(&route)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(0),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let router: RouterData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        tx_builder::build_quote_data(&self.sui_client, quote, &router, &Self::referral_fee(&quote.request)).await
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{AssetId, asset_constants::SUI_USDC_TOKEN_ID};

    const TEST_WALLET: &str = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1";

    #[tokio::test]
    async fn test_cetus_provider_fetch_quote_and_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Cetus::new(rpc_provider);
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

        println!("Cetus quote: {quote:#?}");
        println!("Cetus quote data: {quote_data:#?}");

        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.gas_limit.is_some());

        Ok(())
    }
}
