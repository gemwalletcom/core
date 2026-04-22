use std::collections::HashMap;

use chain_primitives::format_token_id;
use chrono::{DateTime, Utc};
use coingecko::{COINGECKO_CHAIN_MAP, Coin, CoinMarket, get_chain_for_coingecko_platform_id, model::MarketChart};
use primitives::{AssetId, AssetMarket, ChartValue, ChartValuePercentage, Price, PriceProvider};

use crate::{AssetPriceFull, AssetPriceMapping, PriceProviderAsset};

pub fn map_market_chart_daily(chart: MarketChart) -> Vec<ChartValue> {
    chart
        .prices
        .into_iter()
        .filter_map(|p| {
            let ts_ms = *p.first()?;
            let value = *p.get(1)?;
            let ts = DateTime::<Utc>::from_timestamp_millis(ts_ms as i64)?.timestamp() as i32;
            Some(ChartValue {
                timestamp: ts,
                value: value as f32,
            })
        })
        .collect()
}

pub fn map_coin_to_mappings(coin: &Coin) -> Vec<AssetPriceMapping> {
    let mut mappings = Vec::new();
    if let Some(chain) = COINGECKO_CHAIN_MAP.get(coin.id.as_str()) {
        mappings.push(AssetPriceMapping::new(chain.as_asset_id(), coin.id.clone()));
    }
    for (platform_id, contract) in &coin.platforms {
        let Some(chain) = get_chain_for_coingecko_platform_id(platform_id) else {
            continue;
        };
        let Some(contract_address) = contract.as_ref().filter(|a| !a.is_empty()) else {
            continue;
        };
        let Some(token_id) = format_token_id(chain, contract_address.clone()) else {
            continue;
        };
        mappings.push(AssetPriceMapping::new(AssetId::from(chain, Some(token_id)), coin.id.clone()));
    }
    mappings
}

pub fn map_coins_to_mappings(coins: Vec<Coin>) -> Vec<AssetPriceMapping> {
    coins.iter().flat_map(map_coin_to_mappings).collect()
}

pub fn map_coins_to_assets(coins: Vec<Coin>, markets_by_id: HashMap<String, CoinMarket>) -> Vec<PriceProviderAsset> {
    coins
        .iter()
        .flat_map(|coin| {
            let market = markets_by_id.get(&coin.id).map(coin_market_to_asset_market);
            map_coin_to_mappings(coin).into_iter().map(move |mapping| PriceProviderAsset::new(mapping, market.clone()))
        })
        .collect()
}

pub fn map_coin_markets(markets: Vec<CoinMarket>, by_id: &HashMap<String, AssetPriceMapping>) -> Vec<AssetPriceFull> {
    markets
        .into_iter()
        .filter_map(|market| by_id.get(&market.id).cloned().map(|mapping| map_coin_market(market, mapping)))
        .collect()
}

pub fn map_coin_market(market: CoinMarket, mapping: AssetPriceMapping) -> AssetPriceFull {
    let updated_at = market.last_updated.unwrap_or_else(Utc::now);
    let price = market.current_price.unwrap_or_default();
    let market_data = coin_market_to_asset_market(&market);
    AssetPriceFull::new(
        mapping,
        Price::new(price, market.price_change_percentage_24h.unwrap_or_default(), updated_at, PriceProvider::Coingecko),
        Some(market_data),
    )
}

pub fn coin_market_to_asset_market(market: &CoinMarket) -> AssetMarket {
    let price = market.current_price.unwrap_or_default();
    let ath = market.ath.unwrap_or_default();
    let atl = market.atl.unwrap_or_default();
    let ath_percentage = (ath != 0.0).then(|| (price - ath) / ath * 100.0);
    let atl_percentage = (atl != 0.0).then(|| (price - atl) / atl * 100.0);

    AssetMarket {
        market_cap: market.market_cap,
        market_cap_fdv: market.fully_diluted_valuation,
        market_cap_rank: market.effective_market_cap_rank().filter(|&r| r > 0),
        total_volume: market.total_volume,
        circulating_supply: market.circulating_supply,
        total_supply: market.total_supply,
        max_supply: market.max_supply,
        all_time_high: market.ath,
        all_time_high_date: market.ath_date,
        all_time_high_change_percentage: ath_percentage,
        all_time_low: market.atl,
        all_time_low_date: market.atl_date,
        all_time_low_change_percentage: atl_percentage,
        all_time_high_value: market.ath_date.map(|date| ChartValuePercentage {
            date,
            value: ath as f32,
            percentage: ath_percentage.unwrap_or_default() as f32,
        }),
        all_time_low_value: market.atl_date.map(|date| ChartValuePercentage {
            date,
            value: atl as f32,
            percentage: atl_percentage.unwrap_or_default() as f32,
        }),
    }
}
