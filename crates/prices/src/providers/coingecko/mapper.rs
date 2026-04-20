use chrono::Utc;
use coingecko::CoinMarket;
use primitives::{AssetMarket, ChartValuePercentage, Price, PriceProvider};

use crate::{AssetPriceFull, AssetPriceMapping};

pub fn map_coin_market(market: CoinMarket, mapping: AssetPriceMapping) -> AssetPriceFull {
    let updated_at = market.last_updated.unwrap_or_else(Utc::now);
    let price = market.current_price.unwrap_or_default();
    let ath = market.ath.unwrap_or_default();
    let atl = market.atl.unwrap_or_default();
    let ath_percentage = (ath != 0.0).then(|| (price - ath) / ath * 100.0);
    let atl_percentage = (atl != 0.0).then(|| (price - atl) / atl * 100.0);

    let market_data = AssetMarket {
        market_cap: market.market_cap,
        market_cap_fdv: market.fully_diluted_valuation,
        market_cap_rank: market.market_cap_rank,
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
    };

    AssetPriceFull::new(
        mapping,
        Price::new(price, market.price_change_percentage_24h.unwrap_or_default(), updated_at, PriceProvider::Coingecko),
        Some(market_data),
    )
}
