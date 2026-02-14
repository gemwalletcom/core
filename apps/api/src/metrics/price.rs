use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::{AtomicI64, AtomicU64};
use storage::Database;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct PriceLabels {
    asset_id: String,
}

pub fn register(registry: &mut Registry, database: &Database) {
    let assets_total = Gauge::<i64, AtomicI64>::default();
    let updated_at = Family::<PriceLabels, Gauge>::default();
    let value = Family::<PriceLabels, Gauge<f64, AtomicU64>>::default();

    if let Ok(mut client) = database.client() {
        if let Ok(prices) = client.prices().get_prices() {
            assets_total.set(prices.len() as i64);

            for price in prices.iter().filter(|p| p.market_cap_rank >= 1 && p.market_cap_rank <= 10) {
                let labels = PriceLabels { asset_id: price.id.clone() };
                updated_at.get_or_create(&labels).set(price.last_updated_at.and_utc().timestamp());
                value.get_or_create(&labels).set(price.price);
            }
        }
    }

    registry.register("price_assets_total", "Total number of tracked price assets", assets_total);
    registry.register("price_updated_at", "Price last updated timestamp", updated_at);
    registry.register("price_value", "Current price value", value);
}
