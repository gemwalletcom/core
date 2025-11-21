use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use storage::Database;

static PRICE_UPDATED_AT: OnceLock<Family<PriceLabels, Gauge>> = OnceLock::new();
static PRICE_VALUE: OnceLock<Family<PriceLabels, Gauge<f64, AtomicU64>>> = OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct PriceLabels {
    asset_id: String,
}

pub fn init_price_metrics(registry: &mut Registry) {
    let updated_at = Family::<PriceLabels, Gauge>::default();
    let value = Family::<PriceLabels, Gauge<f64, AtomicU64>>::default();

    registry.register("price_updated_at", "Price updated at", updated_at.clone());
    registry.register("price_value", "Price value", value.clone());

    PRICE_UPDATED_AT.set(updated_at).ok();
    PRICE_VALUE.set(value).ok();
}

pub fn update_price_metrics(database: &Database) {
    let prices = database
        .client()
        .ok()
        .and_then(|mut c| c.prices().get_prices().ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|x| x.market_cap_rank >= 1 && x.market_cap_rank <= 10);

    for price in prices {
        if let Some(updated_at) = PRICE_UPDATED_AT.get() {
            updated_at
                .get_or_create(&PriceLabels { asset_id: price.clone().id })
                .set(price.last_updated_at.and_utc().timestamp());
        }
        if let Some(value) = PRICE_VALUE.get() {
            value.get_or_create(&PriceLabels { asset_id: price.clone().id }).set(price.price);
        }
    }
}
