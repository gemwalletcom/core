use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use gem_hypercore::{
    core::{actions::agent::order::make_market_order, hypercore::place_order_typed_data},
    models::{
        spot::{OrderbookResponse, SpotMarket, SpotMeta},
        token::SpotToken,
    },
    rpc::client::HyperCoreClient,
};
use number_formatter::BigNumberFormatter;
use primitives::Chain;

use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset,
    SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    asset::{HYPERCORE_HYPE, HYPERCORE_SPOT_HYPE, HYPERCORE_SPOT_USDC},
};

mod models;
mod simulator;

use models::{SpotRouteData, SpotSide};
use simulator::{simulate_buy, simulate_sell};

const SPOT_META_TTL: Duration = Duration::from_secs(30);
const PAIR_BASE_SYMBOL: &str = "HYPE";
const PAIR_QUOTE_SYMBOL: &str = "USDC";
const MAX_DECIMAL_SCALE: u32 = 6;

#[derive(Debug)]
pub struct HyperCoreSpot {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    client: Mutex<Option<Arc<HyperCoreClient<RpcClient>>>>,
    spot_meta_cache: Mutex<Option<SpotMetaCache>>,
}

#[derive(Debug, Clone)]
struct SpotMetaCache {
    meta: SpotMeta,
    fetched_at: Instant,
}

impl HyperCoreSpot {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperliquid),
            rpc_provider,
            client: Mutex::new(None),
            spot_meta_cache: Mutex::new(None),
        }
    }

    fn client(&self) -> Result<Arc<HyperCoreClient<RpcClient>>, SwapperError> {
        if let Some(client) = self.client.lock().unwrap().as_ref() {
            return Ok(client.clone());
        }

        let endpoint = self.rpc_provider.get_endpoint(Chain::HyperCore)?;
        let client = Arc::new(HyperCoreClient::new(RpcClient::new(endpoint, self.rpc_provider.clone())));
        *self.client.lock().unwrap() = Some(client.clone());
        Ok(client)
    }

    async fn load_spot_meta(&self) -> Result<SpotMeta, SwapperError> {
        if let Some(cache) = self.spot_meta_cache.lock().unwrap().as_ref()
            && cache.fetched_at.elapsed() < SPOT_META_TTL
        {
            return Ok(cache.meta.clone());
        }

        let client = self.client()?;
        let meta = client.get_spot_meta().await.map_err(|err| SwapperError::NetworkError(err.to_string()))?;

        let mut cache = self.spot_meta_cache.lock().unwrap();
        *cache = Some(SpotMetaCache {
            meta: meta.clone(),
            fetched_at: Instant::now(),
        });

        Ok(meta)
    }

    async fn load_orderbook(&self, coin: &str) -> Result<OrderbookResponse, SwapperError> {
        let client = self.client()?;
        client.get_spot_orderbook(coin).await.map_err(|err| SwapperError::NetworkError(err.to_string()))
    }

    fn resolve_token<'a>(&self, meta: &'a SpotMeta, asset: &'a SwapperQuoteAsset) -> Result<&'a SpotToken, SwapperError> {
        let asset_id = asset.asset_id();
        let components = asset_id.token_components().or_else(|| {
            if asset_id == HYPERCORE_HYPE.id {
                Some(("HYPE".to_string(), None, None))
            } else {
                None
            }
        });

        let (symbol, contract, index) = components.ok_or(SwapperError::NotSupportedAsset)?;
        let token = meta
            .tokens()
            .iter()
            .find(|token| token.name.eq_ignore_ascii_case(&symbol))
            .ok_or(SwapperError::NotSupportedAsset)?;

        if let Some(contract) = contract
            && token.token_id != contract
        {
            return Err(SwapperError::NotSupportedAsset);
        }
        if let Some(index) = index
            && token.index != index
        {
            return Err(SwapperError::NotSupportedAsset);
        }

        Ok(token)
    }

    fn resolve_market<'a>(&self, meta: &'a SpotMeta, base: &SpotToken, quote: &SpotToken) -> Result<&'a SpotMarket, SwapperError> {
        meta.universe()
            .iter()
            .find(|market| market.tokens.len() == 2 && market.tokens[0] == base.index && market.tokens[1] == quote.index)
            .ok_or(SwapperError::NotSupportedPair)
    }

    fn format_decimal(value: &BigDecimal) -> String {
        BigNumberFormatter::decimal_to_string(value, MAX_DECIMAL_SCALE)
    }
}

#[async_trait]
impl Swapper for HyperCoreSpot {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::Assets(
            Chain::HyperCore,
            vec![HYPERCORE_SPOT_HYPE.id.clone(), HYPERCORE_SPOT_USDC.id.clone()],
        )]
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let meta = self.load_spot_meta().await?;
        let from_token = self.resolve_token(&meta, &request.from_asset)?;
        let to_token = self.resolve_token(&meta, &request.to_asset)?;

        let amount_in = BigNumberFormatter::big_decimal_value(&request.value, from_token.wei_decimals as u32)?;
        if amount_in <= BigDecimal::zero() {
            return Err(SwapperError::InvalidAmount("amount must be greater than zero".to_string()));
        }

        let (side, base_token, quote_token) = match (from_token.name.as_str(), to_token.name.as_str()) {
            (PAIR_BASE_SYMBOL, PAIR_QUOTE_SYMBOL) => (SpotSide::Sell, from_token, to_token),
            (PAIR_QUOTE_SYMBOL, PAIR_BASE_SYMBOL) => (SpotSide::Buy, to_token, from_token),
            _ => return Err(SwapperError::NotSupportedPair),
        };

        let market = self.resolve_market(&meta, base_token, quote_token)?;
        let orderbook = self.load_orderbook(&base_token.name).await?;
        if orderbook.levels.len() < 2 {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let (output_amount, avg_price) = match side {
            SpotSide::Sell => simulate_sell(&amount_in, &orderbook.levels[0])?,
            SpotSide::Buy => simulate_buy(&amount_in, &orderbook.levels[1])?,
        };

        let decimals_u32: u32 = to_token
            .wei_decimals
            .try_into()
            .map_err(|_| SwapperError::InvalidAmount("invalid amount precision".to_string()))?;

        let output_amount_str = Self::format_decimal(&output_amount);
        let to_value = BigNumberFormatter::value_from_amount(&output_amount_str, decimals_u32)
            .map_err(|err| SwapperError::InvalidAmount(format!("invalid amount: {err}")))?;

        let formatted_amount = Self::format_decimal(&amount_in);
        let avg_price = Self::format_decimal(&avg_price);

        let (size_str, quote_amount_str) = match side {
            SpotSide::Sell => (formatted_amount.clone(), output_amount_str.clone()),
            SpotSide::Buy => (output_amount_str.clone(), formatted_amount.clone()),
        };

        let route_data = SpotRouteData {
            market_index: market.index,
            side: side.clone(),
            size: size_str,
            price: avg_price,
            quote_amount: quote_amount_str,
        };

        let quote = Quote {
            from_value: request.value.clone(),
            to_value,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data).map_err(|err| SwapperError::ComputeQuoteError(err.to_string()))?,
                    gas_limit: None,
                }],
            },
            request: request.clone(),
            eta_in_seconds: None,
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: SpotRouteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let order = make_market_order(
            route_data.market_index,
            route_data.side.is_buy(),
            &route_data.price,
            &route_data.size,
            false,
            None,
        );

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SwapperError::TransactionError("time went backwards".to_string()))?
            .as_millis() as u64;

        let typed_data = place_order_typed_data(order, nonce);

        Ok(SwapperQuoteData::new_contract(
            "".to_string(),
            quote.request.value.clone(),
            typed_data,
            None,
            None,
        ))
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperProvider, SwapperQuoteAsset, testkit::mock_quote};
    use number_formatter::BigNumberFormatter;
    use primitives::swap::SwapQuoteDataType;
    use std::str::FromStr;

    fn native_provider() -> Arc<crate::NativeProvider> {
        Arc::new(crate::NativeProvider::new())
    }

    #[tokio::test]
    async fn test_fetch_spot_quote() {
        let provider = native_provider();
        let spot = HyperCoreSpot::new(provider);

        let from_asset = SwapperQuoteAsset {
            id: HYPERCORE_SPOT_HYPE.id.to_string(),
            symbol: "HYPE".into(),
            decimals: 8,
        };
        let to_asset = SwapperQuoteAsset {
            id: HYPERCORE_SPOT_USDC.id.to_string(),
            symbol: "USDC".into(),
            decimals: 8,
        };

        let mut quote_request = mock_quote(from_asset, to_asset);
        quote_request.options.preferred_providers = vec![SwapperProvider::Hyperliquid];

        let quote = spot.fetch_quote(&quote_request).await.unwrap();
        println!("HyperCoreSpot quote: {:?}", quote);

        let quote_data = spot.fetch_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote.data.provider.id, SwapperProvider::Hyperliquid);
        assert!(!quote.to_value.is_empty());
        assert!(matches!(quote_data.data_type, SwapQuoteDataType::Contract));
        assert!(!quote_data.data.is_empty());

        let base_amount_str = BigNumberFormatter::value(&quote.from_value, quote.request.from_asset.decimals as i32).unwrap();
        let quote_amount_str = BigNumberFormatter::value(&quote.to_value, quote.request.to_asset.decimals as i32).unwrap();

        let base_amount = bigdecimal::BigDecimal::from_str(&base_amount_str).unwrap();
        let quote_amount = bigdecimal::BigDecimal::from_str(&quote_amount_str).unwrap();

        if !base_amount.is_zero() {
            let rate = &quote_amount / &base_amount;
            println!(
                "HyperCoreSpot swap {} {} -> {} {} at rate {}",
                base_amount, quote.request.from_asset.symbol, quote_amount, quote_request.to_asset.symbol, rate
            );
        }
    }
}
