use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use gem_hypercore::{
    core::actions::agent::order::{Builder, PlaceOrder, make_market_order},
    models::{
        spot::{OrderbookResponse, SpotMarket, SpotMeta},
        token::SpotToken,
    },
    rpc::client::HyperCoreClient,
};
use num_bigint::BigUint;
use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::Chain;

use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    asset::{HYPERCORE_HYPE, HYPERCORE_SPOT_HYPE, HYPERCORE_SPOT_UBTC, HYPERCORE_SPOT_USDC},
    error::INVALID_AMOUNT,
};

use super::{
    math::{SpotSide, apply_slippage, format_decimal, format_decimal_with_scale, format_order_size, round_size_down, scale_units, spot_asset_index},
    simulator::{simulate_buy, simulate_sell},
};

const MIN_QUOTE_AMOUNT: i64 = 10;

fn compute_actual_from(use_max_amount: bool, amount: &str, decimals: u32) -> Result<Option<BigUint>, NumberFormatterError> {
    if !use_max_amount {
        return Ok(None);
    }
    BigNumberFormatter::value_from_amount_biguint(amount, decimals).map(Some)
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
        client.get_spot_meta().await.map_err(|err| SwapperError::ComputeQuoteError(err.to_string()))
    }

    async fn load_orderbook(&self, coin: &str) -> Result<OrderbookResponse, SwapperError> {
        let client = self.client()?;
        client.get_spot_orderbook(coin).await.map_err(|err| SwapperError::ComputeQuoteError(err.to_string()))
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

    fn find_direct_market<'a>(
        &self,
        meta: &'a SpotMeta,
        from_token: &'a SpotToken,
        to_token: &'a SpotToken,
    ) -> Result<(&'a SpotMarket, &'a SpotToken, &'a SpotToken, SpotSide), SwapperError> {
        for market in meta.universe().iter().filter(|m| m.tokens.len() == 2) {
            if market.tokens[0] == from_token.index && market.tokens[1] == to_token.index {
                return Ok((market, from_token, to_token, SpotSide::Sell));
            }
            if market.tokens[0] == to_token.index && market.tokens[1] == from_token.index {
                return Ok((market, to_token, from_token, SpotSide::Buy));
            }
        }
        Err(SwapperError::NotSupportedAsset)
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
            vec![HYPERCORE_SPOT_HYPE.id.clone(), HYPERCORE_SPOT_USDC.id.clone(), HYPERCORE_SPOT_UBTC.id.clone()],
        )]
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let client = self.client()?;
        let meta = self.load_spot_meta().await?;
        let from_token = self.resolve_token(&meta, &request.from_asset)?;
        let to_token = self.resolve_token(&meta, &request.to_asset)?;

        let amount_in = BigNumberFormatter::big_decimal_value(&request.value, request.from_asset.decimals)?;
        if amount_in <= BigDecimal::zero() {
            return Err(SwapperError::ComputeQuoteError("amount must be greater than zero".into()));
        }

        let (market, base_token, _quote_token, side) = self.find_direct_market(&meta, from_token, to_token)?;
        let coin = format!("@{}", market.index);
        let orderbook = self.load_orderbook(&coin).await?;
        if orderbook.levels.len() < 2 {
            return Err(SwapperError::NoQuoteAvailable);
        }

        // Round to sz_decimals before simulation to ensure quote matches execution.
        let (raw_output, base_limit_price, size_rounded, actual_from_value) = match side {
            SpotSide::Sell => {
                let rounded_input = round_size_down(&amount_in, base_token.sz_decimals);
                if rounded_input <= BigDecimal::zero() {
                    return Err(SwapperError::ComputeQuoteError("amount too small after rounding".into()));
                }
                let result = simulate_sell(&rounded_input, &orderbook.levels[0])?;
                let actual_from = compute_actual_from(request.options.use_max_amount, &format_decimal(&rounded_input), request.from_asset.decimals)?;
                (result.amount_out, result.limit_price, rounded_input, actual_from)
            }
            SpotSide::Buy => {
                let result = simulate_buy(&amount_in, &orderbook.levels[1])?;
                let rounded_output = round_size_down(&result.amount_out, base_token.sz_decimals);
                if rounded_output <= BigDecimal::zero() {
                    return Err(SwapperError::ComputeQuoteError("output too small after rounding".into()));
                }
                let actual_from = compute_actual_from(
                    request.options.use_max_amount,
                    &format_decimal(&(&rounded_output * &result.limit_price)),
                    request.from_asset.decimals,
                )?;
                (rounded_output.clone(), result.limit_price, rounded_output, actual_from)
            }
        };

        // Check minimum USD value (quote token is USDC)
        let quote_amount = match side {
            SpotSide::Sell => &raw_output,
            SpotSide::Buy => &amount_in,
        };
        if quote_amount < &BigDecimal::from(MIN_QUOTE_AMOUNT) {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }

        let builder_fee = client.config.max_builder_fee_bps;
        let fee_factor = BigDecimal::from(100_000 - builder_fee as i64) / BigDecimal::from(100_000);
        let output_amount = &raw_output * fee_factor;

        let token_decimals: u32 = to_token
            .wei_decimals
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError(format!("{} precision: {}", INVALID_AMOUNT, to_token.wei_decimals)))?;

        let token_units = BigNumberFormatter::value_from_amount_biguint(&format_decimal(&output_amount), token_decimals)
            .map_err(|err| SwapperError::ComputeQuoteError(format!("{}: {err}", INVALID_AMOUNT)))?;
        let scaled_units = scale_units(token_units, token_decimals, request.to_asset.decimals)?;
        let to_value = scaled_units.to_string();

        let price_decimals = 8u32.saturating_sub(base_token.sz_decimals);
        let limit_price = apply_slippage(&base_limit_price, side, request.options.slippage.bps, price_decimals)?;
        let limit_price = format_decimal_with_scale(&limit_price, price_decimals);

        let order_size = format_order_size(&size_rounded, base_token.sz_decimals);

        let asset_index = spot_asset_index(market.index);

        // Adjust from_value for use_max_amount to reflect actual swapped amount after sz_decimals rounding.
        let from_value = actual_from_value.map(|v| v.to_string()).unwrap_or_else(|| request.value.clone());

        let quote = Quote {
            from_value,
            to_value,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&make_market_order(
                        asset_index,
                        side.is_buy(),
                        &limit_price,
                        &order_size,
                        false,
                        Some(Builder {
                            builder_address: client.config.builder_address.clone(),
                            fee: client.config.max_builder_fee_bps,
                        }),
                    ))
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
        let order_json = serde_json::to_string(&order).map_err(|err| SwapperError::TransactionError(err.to_string()))?;

        Ok(SwapperQuoteData::new_contract("".to_string(), quote.request.value.clone(), order_json, None, None))
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::{hyperliquid::provider::spot::math::SPOT_ASSET_OFFSET, testkit::mock_quote};
    use primitives::swap::SwapQuoteDataType;
    use std::str::FromStr;

    fn quote_asset(asset: &primitives::Asset) -> SwapperQuoteAsset {
        SwapperQuoteAsset {
            id: asset.id.to_string(),
            symbol: asset.symbol.clone(),
            decimals: asset.decimals as u32,
        }
    }

    async fn assert_spot_quote(from_asset: SwapperQuoteAsset, to_asset: SwapperQuoteAsset) {
        let spot = HyperCoreSpot::new(Arc::new(crate::NativeProvider::new()));

        let mut request = mock_quote(from_asset, to_asset);
        request.options.preferred_providers = vec![SwapperProvider::Hyperliquid];
        request.value = "2000000000".into();

        let quote = spot.fetch_quote(&request).await.unwrap();

        let order: PlaceOrder = serde_json::from_str(&quote.data.routes[0].route_data).unwrap();
        assert_eq!(order.r#type, "order");
        assert!(order.orders[0].asset >= SPOT_ASSET_OFFSET);

        let quote_data = spot.fetch_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(quote.data.provider.id, SwapperProvider::Hyperliquid);
        assert!(!quote.to_value.is_empty());
        assert!(matches!(quote_data.data_type, SwapQuoteDataType::Contract));

        let from_amount = BigDecimal::from_str(&BigNumberFormatter::value(&quote.from_value, quote.request.from_asset.decimals as i32).unwrap()).unwrap();
        let to_amount = BigDecimal::from_str(&BigNumberFormatter::value(&quote.to_value, quote.request.to_asset.decimals as i32).unwrap()).unwrap();

        assert!(!from_amount.is_zero());
        assert!(!to_amount.is_zero());

        println!(
            "HyperCoreSpot: {} {} -> {} {} (rate: {})",
            from_amount,
            quote.request.from_asset.symbol,
            to_amount,
            quote.request.to_asset.symbol,
            &to_amount / &from_amount
        );
    }

    #[tokio::test]
    async fn test_spot_quote_hype_usdc() {
        assert_spot_quote(quote_asset(&HYPERCORE_SPOT_HYPE), quote_asset(&HYPERCORE_SPOT_USDC)).await;
        assert_spot_quote(quote_asset(&HYPERCORE_SPOT_USDC), quote_asset(&HYPERCORE_SPOT_HYPE)).await;
    }

    #[tokio::test]
    async fn test_spot_quote_ubtc_usdc() {
        assert_spot_quote(quote_asset(&HYPERCORE_SPOT_UBTC), quote_asset(&HYPERCORE_SPOT_USDC)).await;
        assert_spot_quote(quote_asset(&HYPERCORE_SPOT_USDC), quote_asset(&HYPERCORE_SPOT_UBTC)).await;
    }
}
