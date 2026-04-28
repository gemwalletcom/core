use std::collections::HashMap;

use chain_primitives::format_token_id;
use chrono::{DateTime, Utc};
use coingecko::{Coin, CoinInfo, CoinMarket, get_chain_for_coingecko_platform_id, get_chains_for_coingecko_market_id, model::MarketChart};
use primitives::{AssetId, AssetLink, AssetMarket, ChartValue, ChartValuePercentage, LinkType, Price, PriceProvider};

use crate::{AssetPriceFull, AssetPriceMapping, PriceProviderAsset, PriceProviderAssetMetadata};

pub fn map_market_chart(chart: MarketChart) -> Vec<ChartValue> {
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

pub fn map_coin_mappings(id: &str, platforms: &HashMap<String, Option<String>>) -> Vec<AssetPriceMapping> {
    let chains = get_chains_for_coingecko_market_id(id)
        .into_iter()
        .map(|chain| AssetPriceMapping::new(chain.as_asset_id(), id.to_string()));
    let tokens = platforms.iter().filter_map(|(platform_id, contract)| {
        let chain = get_chain_for_coingecko_platform_id(platform_id)?;
        let contract_address = contract.as_ref().filter(|a| !a.is_empty())?;
        let token_id = format_token_id(chain, contract_address.clone())?;
        Some(AssetPriceMapping::new(AssetId::from(chain, Some(token_id)), id.to_string()))
    });
    chains.chain(tokens).collect()
}

pub fn map_coins_to_mappings(coins: Vec<Coin>) -> Vec<AssetPriceMapping> {
    coins.iter().flat_map(|coin| map_coin_mappings(&coin.id, &coin.platforms)).collect()
}

pub fn map_coins_to_assets(coins: Vec<Coin>, markets_by_id: HashMap<String, CoinMarket>) -> Vec<PriceProviderAsset> {
    coins
        .iter()
        .flat_map(|coin| {
            let raw = markets_by_id.get(&coin.id);
            let market = raw.map(coin_market_to_asset_market);
            let price = raw.and_then(|m| m.current_price);
            let price_change_24h = raw.and_then(|m| m.price_change_percentage_24h);
            map_coin_mappings(&coin.id, &coin.platforms)
                .into_iter()
                .map(move |mapping| PriceProviderAsset::with_price(mapping, market.clone(), price, price_change_24h))
        })
        .collect()
}

pub fn map_coin_markets(markets: Vec<CoinMarket>, by_id: &HashMap<String, Vec<AssetPriceMapping>>) -> Vec<AssetPriceFull> {
    markets
        .into_iter()
        .flat_map(|market| {
            by_id
                .get(&market.id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(move |mapping| map_coin_market(market.clone(), mapping))
        })
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

pub fn map_coin_info_metadata(mappings: Vec<AssetPriceMapping>, coin_info: CoinInfo) -> Vec<PriceProviderAssetMetadata> {
    let links = map_coin_info_links(&coin_info);
    mappings
        .into_iter()
        .map(|mapping| {
            let rank = compute_asset_rank(&mapping.asset_id, &coin_info);
            PriceProviderAssetMetadata::new(mapping.asset_id, rank, links.clone())
        })
        .collect()
}

fn compute_asset_rank(asset_id: &AssetId, coin_info: &CoinInfo) -> i32 {
    if asset_id.token_id.is_none() {
        return asset_id.chain.rank();
    }

    let mut rank = 12;
    rank += market_cap_rank_score(coin_info.effective_market_cap_rank().unwrap_or_default());
    rank += platform_diversity_score(coin_info.platforms.len());
    rank += social_score(coin_info);
    rank += asset_id.chain.rank() / 20;
    rank.max(16)
}

fn market_cap_rank_score(market_cap_rank: i32) -> i32 {
    match market_cap_rank {
        1..25 => 15,
        25..50 => 12,
        50..100 => 10,
        100..250 => 8,
        250..500 => 6,
        500..1000 => 4,
        1000..2000 => 2,
        2000..4000 => 0,
        4000..5000 => -1,
        _ => -2,
    }
}

fn platform_diversity_score(platform_count: usize) -> i32 {
    if platform_count > 6 {
        2
    } else if platform_count > 3 {
        1
    } else {
        0
    }
}

fn social_score(coin_info: &CoinInfo) -> i32 {
    let twitter_score = coin_info
        .community_data
        .as_ref()
        .filter(|d| d.twitter_followers.unwrap_or_default() > 128_000)
        .map(|_| 1)
        .unwrap_or_default();

    let watchlist = coin_info.watchlist_portfolio_users.unwrap_or_default() as i32;
    let watchlist_score = if watchlist > 1_000_000 {
        2
    } else if watchlist > 250_000 {
        1
    } else {
        0
    };

    twitter_score + watchlist_score
}

fn map_coin_info_links(coin_info: &CoinInfo) -> Vec<AssetLink> {
    let links = &coin_info.links;
    let mut results = vec![AssetLink::new(
        &format!("https://www.coingecko.com/coins/{}", coin_info.id.to_lowercase()),
        LinkType::Coingecko,
    )];

    if let Some(value) = links.twitter_screen_name.as_ref().filter(|v| !v.is_empty()) {
        results.push(AssetLink::new(&format!("https://x.com/{value}"), LinkType::X));
    }

    if let Some(value) = links.homepage.iter().find(|x| !x.is_empty()).filter(|v| !v.starts_with("https://t.me")) {
        results.push(AssetLink::new(value, LinkType::Website));
    }

    if let Some(value) = links.telegram_channel_identifier.as_ref().filter(|v| !v.is_empty()) {
        results.push(AssetLink::new(&format!("https://t.me/{value}"), LinkType::Telegram));
    }

    if let Some(value) = links.chat_url.iter().find(|x| x.contains("discord.com")) {
        results.push(AssetLink::new(value, LinkType::Discord));
    }

    if let Some(value) = links.repos_url.get("github").and_then(|urls| urls.iter().find(|x| !x.is_empty())) {
        results.push(AssetLink::new(value, LinkType::GitHub));
    }

    results
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use coingecko::CoinMarket;
    use primitives::Chain;

    use super::*;

    #[test]
    fn test_map_coin_markets_preserves_platform_mappings() {
        let provider_price_id = "atua-ai".to_string();
        let ethereum = AssetPriceMapping::new(
            AssetId::from_token(Chain::Ethereum, "0x791A5c2261823dBF69b27B63E851B7745532Cfa2"),
            provider_price_id.clone(),
        );
        let smartchain = AssetPriceMapping::new(
            AssetId::from_token(Chain::SmartChain, "0x36b2269FD151208a4bfc3DEA503E0a6F2485fA78"),
            provider_price_id.clone(),
        );
        let by_id = HashMap::from([(provider_price_id.clone(), vec![ethereum.clone(), smartchain.clone()])]);

        let prices = map_coin_markets(vec![CoinMarket::mock_with_id(&provider_price_id)], &by_id);

        assert_eq!(prices.len(), 2);
        assert_eq!(prices[0].mapping.asset_id, ethereum.asset_id);
        assert_eq!(prices[0].mapping.provider_price_id, ethereum.provider_price_id);
        assert_eq!(prices[1].mapping.asset_id, smartchain.asset_id);
        assert_eq!(prices[1].mapping.provider_price_id, smartchain.provider_price_id);
        assert_eq!(prices[0].price.price, 0.12);
        assert_eq!(prices[1].price.price, 0.12);
    }
}
