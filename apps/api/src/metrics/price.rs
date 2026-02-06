use cacher::{CacheKey, CacherClient};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use storage::Database;

static PRICE_UPDATED_AT: OnceLock<Family<PriceLabels, Gauge>> = OnceLock::new();
static PRICE_VALUE: OnceLock<Family<PriceLabels, Gauge<f64, AtomicU64>>> = OnceLock::new();
static PRICE_ASSETS_TOTAL: OnceLock<Gauge> = OnceLock::new();
static PRICE_OBSERVED_ASSETS_TOTAL: OnceLock<Gauge> = OnceLock::new();
static PRICE_OBSERVED_ASSET_SCORE: OnceLock<Family<PriceLabels, Gauge<f64, AtomicU64>>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct PriceLabels {
    asset_id: String,
}

pub fn init_price_metrics(registry: &mut Registry) {
    let updated_at = Family::<PriceLabels, Gauge>::default();
    let value = Family::<PriceLabels, Gauge<f64, AtomicU64>>::default();
    let assets_total = Gauge::default();
    let observed_total = Gauge::default();
    let observed_score = Family::<PriceLabels, Gauge<f64, AtomicU64>>::default();

    registry.register("price_updated_at", "Price updated at", updated_at.clone());
    registry.register("price_value", "Price value", value.clone());
    registry.register("price_assets_total", "Total number of tracked price assets", assets_total.clone());
    registry.register("price_observed_assets_total", "Total unique assets being observed via WebSocket", observed_total.clone());
    registry.register("price_observed_asset_score", "Observer count for top observed assets", observed_score.clone());

    PRICE_UPDATED_AT.set(updated_at).ok();
    PRICE_VALUE.set(value).ok();
    PRICE_ASSETS_TOTAL.set(assets_total).ok();
    PRICE_OBSERVED_ASSETS_TOTAL.set(observed_total).ok();
    PRICE_OBSERVED_ASSET_SCORE.set(observed_score).ok();
}

pub async fn update_price_metrics(database: &Database, cacher: &CacherClient) {
    update_db_price_metrics(database);
    update_observed_metrics(cacher).await;
}

fn update_db_price_metrics(database: &Database) {
    let prices = database
        .client()
        .ok()
        .and_then(|mut c| c.prices().get_prices().ok())
        .unwrap_or_default();

    if let Some(gauge) = PRICE_ASSETS_TOTAL.get() {
        gauge.set(prices.len() as i64);
    }

    for price in prices.into_iter().filter(|x| x.market_cap_rank >= 1 && x.market_cap_rank <= 10) {
        let labels = PriceLabels { asset_id: price.id.clone() };
        if let Some(family) = PRICE_UPDATED_AT.get() {
            family.get_or_create(&labels).set(price.last_updated_at.and_utc().timestamp());
        }
        if let Some(family) = PRICE_VALUE.get() {
            family.get_or_create(&labels).set(price.price);
        }
    }
}

async fn update_observed_metrics(cacher: &CacherClient) {
    let key = CacheKey::ObservedAssets;

    if let Ok(count) = cacher.sorted_set_card(&key.key()).await
        && let Some(gauge) = PRICE_OBSERVED_ASSETS_TOTAL.get()
    {
        gauge.set(count as i64);
    }

    if let Some(family) = PRICE_OBSERVED_ASSET_SCORE.get() {
        family.clear();
        if let Ok(top_assets) = cacher.sorted_set_rev_range_with_scores(&key.key(), 0, 9).await {
            for (asset_id, score) in top_assets {
                family.get_or_create(&PriceLabels { asset_id }).set(score);
            }
        }
    }
}
