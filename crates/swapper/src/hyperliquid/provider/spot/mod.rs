use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use gem_hypercore::{
    core::actions::agent::order::{PlaceOrder, make_market_order},
    models::{
        spot::{OrderbookResponse, SpotMarket, SpotMeta},
        token::SpotToken,
    },
    rpc::client::HyperCoreClient,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use number_formatter::BigNumberFormatter;
use primitives::Chain;
use std::str::FromStr;

use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset,
    SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    asset::{HYPERCORE_HYPE, HYPERCORE_SPOT_HYPE, HYPERCORE_SPOT_USDC},
};

mod simulator;
use simulator::{SimulationResult, simulate_buy, simulate_sell};

const PAIR_BASE_SYMBOL: &str = "HYPE";
const PAIR_QUOTE_SYMBOL: &str = "USDC";
const MAX_DECIMAL_SCALE: u32 = 6;
// HyperLiquid spot assets use `10000 + spotMeta.universe[index]`.
// Doc: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint#asset
const SPOT_ASSET_OFFSET: u32 = 10_000;

#[derive(Debug, Clone, Copy)]
enum SpotSide {
    Buy,
    Sell,
}

impl SpotSide {
    fn is_buy(self) -> bool {
        matches!(self, SpotSide::Buy)
    }
}

#[derive(Debug)]
pub struct HyperCoreSpot {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    client: Mutex<Option<Arc<HyperCoreClient<RpcClient>>>>,
}

impl HyperCoreSpot {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperliquid),
            rpc_provider,
            client: Mutex::new(None),
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
        let client = self.client()?;
        client.get_spot_meta().await.map_err(|err| SwapperError::NetworkError(err.to_string()))
    }

    async fn load_orderbook(&self, coin: &str) -> Result<OrderbookResponse, SwapperError> {
        let client = self.client()?;
        client.get_spot_orderbook(coin).await.map_err(|err| SwapperError::NetworkError(err.to_string()))
    }

    fn resolve_token<'a>(&self, meta: &'a SpotMeta, asset: &'a SwapperQuoteAsset) -> Result<&'a SpotToken, SwapperError> {
        let asset_id = asset.asset_id();
        let components = asset_id.token_components().or_else(|| {
            if asset_id == HYPERCORE_HYPE.id {
                HYPERCORE_SPOT_HYPE.id.token_components()
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
        Self::format_decimal_with_scale(value, MAX_DECIMAL_SCALE)
    }

    fn format_decimal_with_scale(value: &BigDecimal, scale: u32) -> String {
        BigNumberFormatter::decimal_to_string(value, scale)
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

        let amount_in = BigNumberFormatter::big_decimal_value(&request.value, request.from_asset.decimals)?;
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

        let SimulationResult {
            amount_out: output_amount,
            limit_price: base_limit_price,
        } = match side {
            SpotSide::Sell => simulate_sell(&amount_in, &orderbook.levels[0])?,
            SpotSide::Buy => simulate_buy(&amount_in, &orderbook.levels[1])?,
        };

        let token_decimals: u32 = to_token
            .wei_decimals
            .try_into()
            .map_err(|_| SwapperError::InvalidAmount("invalid amount precision".to_string()))?;

        let output_amount_str = Self::format_decimal(&output_amount);
        let token_units = BigNumberFormatter::value_from_amount_biguint(&output_amount_str, token_decimals)
            .map_err(|err| SwapperError::InvalidAmount(format!("invalid amount: {err}")))?;
        let scaled_units = scale_units(token_units, token_decimals, request.to_asset.decimals)?;
        let to_value = scaled_units.to_string();

        let price_decimals = 8u32.saturating_sub(base_token.sz_decimals);
        let limit_price = apply_slippage(&base_limit_price, side, request.options.slippage.bps, price_decimals)?;
        let limit_price = Self::format_decimal_with_scale(&limit_price, price_decimals);

        let size_value = match side {
            SpotSide::Sell => amount_in.clone(),
            SpotSide::Buy => output_amount.clone(),
        };
        let size_str = format_order_size(&size_value, base_token.sz_decimals)?;

        let asset_index = spot_asset_index(market.index);

        let quote = Quote {
            from_value: request.value.clone(),
            to_value,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&make_market_order(asset_index, side.is_buy(), &limit_price, &size_str, false, None))
                        .map_err(|err| SwapperError::ComputeQuoteError(err.to_string()))?,
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
        let order: PlaceOrder = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let order_json = serde_json::to_string(&order).map_err(|err| SwapperError::ComputeQuoteError(err.to_string()))?;

        Ok(SwapperQuoteData::new_contract(
            "".to_string(),
            quote.request.value.clone(),
            order_json,
            None,
            None,
        ))
    }
}

fn scale_units(value: BigUint, from_decimals: u32, to_decimals: u32) -> Result<BigUint, SwapperError> {
    if from_decimals == to_decimals {
        return Ok(value);
    }

    if to_decimals > from_decimals {
        let diff = to_decimals - from_decimals;
        let factor = BigUint::from(10u32).pow(diff);
        Ok(value * factor)
    } else {
        let diff = from_decimals - to_decimals;
        let factor = BigUint::from(10u32).pow(diff);
        let remainder = &value % &factor;
        if remainder != BigUint::from(0u32) {
            return Err(SwapperError::InvalidAmount("amount precision loss".to_string()));
        }
        Ok(value / factor)
    }
}

fn format_order_size(amount: &BigDecimal, decimals: u32) -> Result<String, SwapperError> {
    let value = amount
        .to_f64()
        .ok_or_else(|| SwapperError::InvalidAmount("failed to convert amount".to_string()))?;
    let rounded = round_to_decimals(value, decimals);
    let formatted = if decimals == 0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.decimals$}", decimals = decimals as usize)
    };
    let big_decimal = BigDecimal::from_str(&formatted).map_err(|_| SwapperError::InvalidAmount("failed to format size".to_string()))?;
    Ok(BigNumberFormatter::decimal_to_string(&big_decimal, decimals))
}

fn spot_asset_index(market_index: u32) -> u32 {
    SPOT_ASSET_OFFSET + market_index
}

fn apply_slippage(limit_price: &BigDecimal, side: SpotSide, slippage_bps: u32, price_decimals: u32) -> Result<BigDecimal, SwapperError> {
    if limit_price <= &BigDecimal::zero() {
        return Err(SwapperError::InvalidAmount("invalid limit price".to_string()));
    }

    let limit_price_f64 = limit_price
        .to_f64()
        .ok_or_else(|| SwapperError::InvalidAmount("failed to convert price".to_string()))?;

    let slippage_fraction = slippage_bps as f64 / 10_000.0;
    let multiplier = if side.is_buy() { 1.0 + slippage_fraction } else { 1.0 - slippage_fraction };

    if multiplier <= 0.0 {
        return Err(SwapperError::InvalidAmount("slippage multiplier not positive".to_string()));
    }

    let adjusted = limit_price_f64 * multiplier;
    let rounded = round_to_significant_and_decimal(adjusted, 5, price_decimals);

    let formatted = if price_decimals == 0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.price_decimals$}", price_decimals = price_decimals as usize)
    };

    BigDecimal::from_str(&formatted).map_err(|_| SwapperError::InvalidAmount("failed to format limit price".to_string()))
}

fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

fn round_to_significant_and_decimal(value: f64, sig_figs: u32, max_decimals: u32) -> f64 {
    if value == 0.0 {
        return 0.0;
    }

    let abs_value = value.abs();
    let magnitude = abs_value.log10().floor() as i32;
    let scale = 10f64.powi(sig_figs as i32 - magnitude - 1);
    let rounded = (abs_value * scale).round() / scale;
    round_to_decimals(rounded.copysign(value), max_decimals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use number_formatter::BigNumberFormatter;
    use std::str::FromStr;

    #[test]
    fn test_scale_units_up() {
        let base = BigUint::from(123u32);
        let scaled = scale_units(base.clone(), 8, 18).unwrap();
        let expected = BigUint::from(10u32).pow(10) * base;
        assert_eq!(scaled, expected);
    }

    #[test]
    fn test_scale_units_down() {
        let value = BigUint::from(123u32) * BigUint::from(10u32).pow(10);
        let scaled = scale_units(value.clone(), 18, 8).unwrap();
        assert_eq!(scaled, BigUint::from(123u32));
    }

    #[test]
    fn test_scale_units_down_rejects_remainder() {
        let err = scale_units(BigUint::from(5u32), 3, 1).unwrap_err();
        assert!(matches!(err, SwapperError::InvalidAmount(_)));
    }

    #[test]
    fn test_format_order_size_rounds() {
        let value = BigDecimal::from_str("0.131").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "0.13");

        let value = BigDecimal::from_str("0.189834").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "0.19");

        let value = BigDecimal::from_str("0.10").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "0.1");

        let value = BigDecimal::from_str("-0.131").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "-0.13");
    }

    #[test]
    fn test_spot_asset_index_offset() {
        assert_eq!(spot_asset_index(0), SPOT_ASSET_OFFSET);
        assert_eq!(spot_asset_index(107), SPOT_ASSET_OFFSET + 107);
    }

    #[test]
    fn test_apply_slippage_buy_increases_price() {
        let price = BigDecimal::from_str("100").unwrap();
        let adjusted = apply_slippage(&price, SpotSide::Buy, 1000, 2).unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&adjusted, 2), "110");
    }

    #[test]
    fn test_apply_slippage_sell_decreases_price() {
        let price = BigDecimal::from_str("100").unwrap();
        let adjusted = apply_slippage(&price, SpotSide::Sell, 500, 2).unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&adjusted, 2), "95");
    }

    #[test]
    fn test_apply_slippage_zero_returns_same_price() {
        let price = BigDecimal::from_str("42.123456").unwrap();
        let adjusted = apply_slippage(&price, SpotSide::Sell, 0, 4).unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&adjusted, 4), "42.123");
    }

    #[test]
    fn test_apply_slippage_invalid_when_multiplier_non_positive() {
        let price = BigDecimal::from_str("10").unwrap();
        assert!(apply_slippage(&price, SpotSide::Sell, 10001, 2).is_err());
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperProvider, SwapperQuoteAsset, testkit::mock_quote};
    use number_formatter::BigNumberFormatter;
    use primitives::swap::SwapQuoteDataType;
    use std::str::FromStr;

    const HYPE_SIZE_DECIMALS: u32 = 2;

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

        let order: PlaceOrder = serde_json::from_str(&quote.data.routes[0].route_data).unwrap();
        assert_eq!(order.r#type, "order");
        assert!(order.orders[0].asset >= SPOT_ASSET_OFFSET);
        assert!(order.orders[0].asset - SPOT_ASSET_OFFSET < SPOT_ASSET_OFFSET);
        let expected_size = format_order_size(
            &BigDecimal::from_str(&BigNumberFormatter::value(&quote.from_value, quote.request.from_asset.decimals as i32).unwrap()).unwrap(),
            HYPE_SIZE_DECIMALS,
        )
        .unwrap();
        assert_eq!(order.orders[0].size, expected_size);
        assert_eq!(order.orders[0].size.split('.').nth(1).unwrap().len(), HYPE_SIZE_DECIMALS as usize);

        let quote_data = spot.fetch_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let payload_order: PlaceOrder = serde_json::from_str(&quote_data.data).unwrap();
        assert_eq!(payload_order.orders.len(), order.orders.len());

        assert_eq!(payload_order.orders[0].size, order.orders[0].size);

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
