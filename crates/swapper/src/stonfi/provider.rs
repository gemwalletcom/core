use super::{
    client::StonfiClient,
    model::{QuotePath, SimulateSwapRequest, SwapSimulation},
    tx_builder::{NextSwapParams, ReferralParams, SwapTransactionParams, build_swap_transaction},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset,
    SwapperQuoteData,
    config::get_swap_proxy_url,
    fees::{ReferralFee, default_referral_fees, quote_value_after_reserve_by_chain},
};
use async_trait::async_trait;
use futures::future::{join, join_all};
use gem_client::Client;
use gem_ton::{
    address::{Address, base64_to_hex_address},
    constants::TON_PROXY_JETTON_ADDRESS,
    rpc::client::TonClient,
};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_ASSET_ID};
use std::{fmt::Debug, str::FromStr, sync::Arc};

const SLIPPAGE_BPS_DECIMALS: u32 = 4;

#[derive(Debug)]
pub struct Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: StonfiClient<C>,
    ton_client: TonClient<RpcClient>,
}

impl Stonfi<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Ton).expect("failed to get TON endpoint for STON.fi");
        let ton_client = TonClient::new(RpcClient::new(endpoint, rpc_provider.clone()));
        Self::new_with_clients(RpcClient::new(get_swap_proxy_url("stonfi"), rpc_provider), ton_client)
    }
}

impl<C> Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_clients(client: C, ton_client: TonClient<RpcClient>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::StonfiV2),
            client: StonfiClient::new(client),
            ton_client,
        }
    }

    fn intermediary_tokens() -> Vec<SwapperQuoteAsset> {
        vec![SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)), SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone())]
    }

    async fn quote_path_via_intermediary(&self, intermediary: &SwapperQuoteAsset, from_value: &str, request: &QuoteRequest) -> Result<QuotePath, SwapperError> {
        let referral_fee = Self::referral_fee(request);
        let to_intermediary = self
            .simulate(&request.from_asset, from_value, intermediary, request, ReferralFee { bps: 0, ..referral_fee.clone() })
            .await?;
        if !to_intermediary.router.is_supported_v2() {
            return Err(SwapperError::InvalidRoute);
        }
        // The second hop quote uses the first hop's expected output; execution still applies min_ask_units per hop.
        let from_intermediary = self.simulate(intermediary, &to_intermediary.ask_units, &request.to_asset, request, referral_fee).await?;
        if !from_intermediary.router.is_supported_v2() {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(QuotePath {
            to_value: from_intermediary.ask_units.clone(),
            routes: vec![
                Route {
                    input: request.from_asset.asset_id(),
                    output: intermediary.asset_id(),
                    route_data: serde_json::to_string(&to_intermediary)?,
                },
                Route {
                    input: intermediary.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&from_intermediary)?,
                },
            ],
        })
    }

    async fn quote_direct(&self, request: &QuoteRequest, from_value: &str) -> Result<QuotePath, SwapperError> {
        let simulation = self
            .simulate(&request.from_asset, from_value, &request.to_asset, request, Self::referral_fee(request))
            .await?;
        Ok(QuotePath {
            to_value: simulation.ask_units.clone(),
            routes: vec![Route {
                input: request.from_asset.asset_id(),
                output: request.to_asset.asset_id(),
                route_data: serde_json::to_string(&simulation)?,
            }],
        })
    }

    async fn quote_intermediary_paths(&self, request: &QuoteRequest, from_value: &str) -> Vec<Result<QuotePath, SwapperError>> {
        let intermediary_tokens = Self::intermediary_tokens()
            .into_iter()
            .filter(|x| {
                let intermediary_id = x.asset_id();
                intermediary_id != request.from_asset.asset_id() && intermediary_id != request.to_asset.asset_id()
            })
            .collect::<Vec<_>>();
        join_all(
            intermediary_tokens
                .iter()
                .map(|intermediary| self.quote_path_via_intermediary(intermediary, from_value, request)),
        )
        .await
    }

    fn referral_fee(request: &QuoteRequest) -> ReferralFee {
        request.options.fee.clone().map(|fees| fees.ton).unwrap_or_else(|| default_referral_fees().ton)
    }

    async fn simulate(
        &self,
        from_asset: &SwapperQuoteAsset,
        from_value: &str,
        to_asset: &SwapperQuoteAsset,
        request: &QuoteRequest,
        referral_fee: ReferralFee,
    ) -> Result<SwapSimulation, SwapperError> {
        let simulation_request = SimulateSwapRequest {
            offer_address: token_address(from_asset),
            units: from_value.to_string(),
            ask_address: token_address(to_asset),
            slippage_tolerance: slippage_tolerance(request.options.slippage.bps)?,
            referral_address: referral_fee.address,
            referral_fee_bps: referral_fee.bps.to_string(),
        };
        self.client.simulate_swap(&simulation_request).await
    }

    fn select_best_quote_path(paths: impl IntoIterator<Item = Result<QuotePath, SwapperError>>) -> Result<QuotePath, SwapperError> {
        let mut error = None;
        paths
            .into_iter()
            .filter_map(|result| match result {
                Ok(path) => BigUint::from_str(&path.to_value).ok().map(|amount| (amount, path)),
                Err(err) => {
                    if error.is_none() {
                        error = Some(err);
                    }
                    None
                }
            })
            .max_by(|(left, _), (right, _)| left.cmp(right))
            .map(|(_, path)| path)
            .ok_or_else(|| error.unwrap_or(SwapperError::NoQuoteAvailable))
    }

    async fn get_quotes(&self, request: &QuoteRequest, from_value: &str) -> Result<QuotePath, SwapperError> {
        if request.from_asset.is_native() || request.to_asset.is_native() {
            return self.quote_direct(request, from_value).await;
        }

        let (direct, intermediary_paths) = join(self.quote_direct(request, from_value), self.quote_intermediary_paths(request, from_value)).await;
        Self::select_best_quote_path(std::iter::once(direct).chain(intermediary_paths))
    }

    async fn sender_jetton_wallet(&self, quote: &Quote) -> Result<Option<String>, SwapperError> {
        if quote.request.from_asset.is_native() {
            return Ok(None);
        }
        let token_id = quote.request.from_asset.asset_id().token_id.ok_or(SwapperError::NotSupportedAsset)?;
        let jetton_token_id = base64_to_hex_address(&token_id).ok_or(SwapperError::NotSupportedAsset)?.to_uppercase();
        let wallet = self
            .ton_client
            .get_jetton_wallets(quote.request.wallet_address.clone())
            .await
            .map_err(|err| SwapperError::ComputeQuoteError(err.to_string()))?
            .jetton_wallets
            .into_iter()
            .find(|wallet| wallet.jetton == jetton_token_id)
            .ok_or_else(|| SwapperError::ComputeQuoteError("missing sender jetton wallet".into()))?;
        Ok(Some(wallet.address))
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
        let from_value = quote_value_after_reserve_by_chain(request)?;
        let path = self.get_quotes(request, &from_value).await?;

        Ok(Quote {
            from_value,
            to_value: path.to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: path.routes,
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let simulations = quote
            .data
            .routes
            .iter()
            .map(|route| serde_json::from_str::<SwapSimulation>(&route.route_data).map_err(|_| SwapperError::InvalidRoute))
            .collect::<Result<Vec<_>, _>>()?;
        if simulations.len() > 2 {
            return Err(SwapperError::InvalidRoute);
        }
        let simulation = simulations.first().ok_or(SwapperError::InvalidRoute)?;
        let next_swap = simulations
            .get(1)
            .map(|next_simulation| {
                Ok::<_, SwapperError>(NextSwapParams {
                    simulation: next_simulation,
                    min_ask_amount: scaled_next_min_ask_amount(simulation, next_simulation)?,
                })
            })
            .transpose()?;
        let referral_fee = Self::referral_fee(&quote.request);
        let receiver_address = if quote.request.destination_address.is_empty() {
            &quote.request.wallet_address
        } else {
            &quote.request.destination_address
        };
        let sender_jetton_wallet = self.sender_jetton_wallet(quote).await?;

        let tx = build_swap_transaction(SwapTransactionParams {
            simulation,
            next_swap,
            from_native: quote.request.from_asset.is_native(),
            to_native: quote.request.to_asset.is_native(),
            sender_jetton_wallet: sender_jetton_wallet.as_deref(),
            from_value: &quote.from_value,
            wallet_address: Address::parse(&quote.request.wallet_address)?,
            receiver_address: Address::parse(receiver_address)?,
            referral: ReferralParams {
                address: Address::parse(&referral_fee.address)?,
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

fn scaled_next_min_ask_amount(first: &SwapSimulation, next: &SwapSimulation) -> Result<BigUint, SwapperError> {
    let first_ask = BigUint::from_str(&first.ask_units)?;
    if first_ask == BigUint::from(0u8) {
        return Err(SwapperError::InvalidRoute);
    }
    let first_min = BigUint::from_str(&first.min_ask_units)?;
    let next_min = BigUint::from_str(&next.min_ask_units)?;
    Ok((next_min * first_min) / first_ask)
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

    #[test]
    fn test_scaled_next_min_ask_amount() {
        let first = SwapSimulation::mock("", "", "260238", "257635");
        let next = SwapSimulation::mock("", "", "709", "702");

        assert_eq!(scaled_next_min_ask_amount(&first, &next).unwrap(), BigUint::from(694u32));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        Options, SwapperQuoteAsset,
        alien::reqwest_provider::NativeProvider,
        stonfi::testkit::NOT_TOKEN_ID,
        testkit::mock_ton,
    };
    use primitives::{AssetId, asset_constants::TON_USDT_ASSET_ID, testkit::signer_mock::TEST_TON_SENDER};

    #[tokio::test]
    async fn test_stonfi_fetch_quote_and_quote_data_ton_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider);
        let request = mock_ton(TEST_TON_SENDER.to_string());

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

    #[tokio::test]
    async fn test_stonfi_fetch_quote_and_quote_data_not_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, NOT_TOKEN_ID)),
            to_asset: SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone()),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: "1000000000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 2);
        assert_eq!(quote.data.routes[0].input, AssetId::from_token(Chain::Ton, NOT_TOKEN_ID));
        assert_eq!(quote.data.routes[0].output, AssetId::from_chain(Chain::Ton));
        assert_eq!(quote.data.routes[1].input, AssetId::from_chain(Chain::Ton));
        assert_eq!(quote.data.routes[1].output, TON_USDT_ASSET_ID.clone());
        println!("STON.fi NOT -> USDT quote: {quote:?}");

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.value.is_empty());
        assert!(quote_data.data.starts_with("te6cc"));
        println!("STON.fi NOT -> USDT quote_data: {quote_data:?}");

        Ok(())
    }
}
