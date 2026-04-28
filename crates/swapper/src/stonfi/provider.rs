use super::{
    client::StonfiClient,
    model::{SimulateSwapRequest, SwapSimulation},
    tx_builder::{ReferralParams, SwapTransactionParams, build_swap_transaction},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, Swapper, SwapperChainAsset, SwapperError, SwapperMode, SwapperProvider,
    SwapperQuoteAsset, SwapperQuoteData,
    fees::{default_referral_fees, resolve_max_quote_value},
};
use async_trait::async_trait;
use gem_client::Client;
use gem_ton::constants::TON_PROXY_JETTON_ADDRESS;
use number_formatter::BigNumberFormatter;
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

const STONFI_API_URL: &str = "https://api.ston.fi";
const ETA_IN_SECONDS: u32 = 3;
const SLIPPAGE_BPS_DECIMALS: u32 = 4;

#[derive(Debug)]
pub struct Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: StonfiClient<C>,
}

impl Stonfi<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_client(RpcClient::new(STONFI_API_URL.to_string(), rpc_provider))
    }
}

impl<C> Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(client: C) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::StonfiV2),
            client: StonfiClient::new(client),
        }
    }
}

#[async_trait]
impl<C> Swapper for Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Ton)]
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        match request.mode {
            SwapperMode::ExactIn => {}
            SwapperMode::ExactOut => return Err(SwapperError::NoQuoteAvailable),
        }

        let from_value = resolve_max_quote_value(request)?;
        let referral_fee = request
            .options
            .fee
            .clone()
            .map(|fees| fees.ton)
            .filter(|fee| !fee.address.is_empty())
            .unwrap_or_else(|| default_referral_fees().ton);
        let simulation_request = SimulateSwapRequest {
            offer_address: token_address(&request.from_asset),
            units: from_value.clone(),
            ask_address: token_address(&request.to_asset),
            slippage_tolerance: slippage_tolerance(request.options.slippage.bps)?,
            referral_address: referral_fee.address.clone(),
            referral_fee_bps: referral_fee.bps.to_string(),
        };

        let simulation = self.client.simulate_swap(&simulation_request).await?;

        Ok(Quote {
            from_value,
            to_value: simulation.ask_units.clone(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&simulation)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(ETA_IN_SECONDS),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let simulation: SwapSimulation = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let referral_fee = quote
            .request
            .options
            .fee
            .clone()
            .map(|fees| fees.ton)
            .filter(|fee| !fee.address.is_empty())
            .unwrap_or_else(|| default_referral_fees().ton);
        let receiver_address = if quote.request.destination_address.is_empty() {
            &quote.request.wallet_address
        } else {
            &quote.request.destination_address
        };

        let tx = build_swap_transaction(SwapTransactionParams {
            simulation: &simulation,
            from_native: quote.request.from_asset.is_native(),
            to_native: quote.request.to_asset.is_native(),
            from_value: &quote.from_value,
            min_ask_amount: &simulation.min_ask_units,
            wallet_address: &quote.request.wallet_address,
            receiver_address,
            referral: ReferralParams {
                address: &referral_fee.address,
                bps: referral_fee.bps,
            },
            deadline: None,
        })?;

        Ok(SwapperQuoteData::new_contract(tx.to, tx.value, tx.data, None, None))
    }
}

fn token_address(asset: &SwapperQuoteAsset) -> String {
    let asset_id = asset.asset_id();
    match asset_id.token_id {
        Some(token_id) => token_id,
        None => TON_PROXY_JETTON_ADDRESS.to_string(),
    }
}

fn slippage_tolerance(bps: u32) -> Result<String, SwapperError> {
    Ok(BigNumberFormatter::value(&u64::from(bps).to_string(), SLIPPAGE_BPS_DECIMALS as i32)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, asset_constants::TON_USDT_TOKEN_ID};

    #[test]
    fn test_token_address_and_slippage_tolerance() {
        assert_eq!(token_address(&SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton))), TON_PROXY_JETTON_ADDRESS);
        assert_eq!(
            token_address(&SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID))),
            TON_USDT_TOKEN_ID
        );
        assert_eq!(slippage_tolerance(0).unwrap(), "0");
        assert_eq!(slippage_tolerance(50).unwrap(), "0.005");
        assert_eq!(slippage_tolerance(100).unwrap(), "0.01");
        assert_eq!(slippage_tolerance(10_000).unwrap(), "1");
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{alien::reqwest_provider::NativeProvider, testkit::make_ton};

    #[tokio::test]
    async fn test_stonfi_fetch_quote_and_quote_data_ton_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider);
        let request = make_ton("UQDxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin201".to_string());

        let quote = provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        println!("STON.fi TON -> USDT quote: {quote:?}");

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.value.is_empty());
        assert!(quote_data.data.starts_with("te6cc"));
        println!("STON.fi TON -> USDT quote_data: {quote_data:?}");

        Ok(())
    }
}
