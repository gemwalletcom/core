use std::sync::atomic::AtomicI64;
use std::sync::Mutex;

use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use super::MetricsProvider;

#[derive(Default)]
struct PriceState {
    price_updates: u64,
    prices_updated: u64,
    prices_last_updated_at: Option<u64>,
    fiat_rates_updated: u64,
    fiat_rates_last_updated_at: Option<u64>,
}

pub struct PriceMetrics {
    state: Mutex<PriceState>,
}

impl PriceMetrics {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(PriceState::default()),
        }
    }

    pub fn record_prices_update(&self, count: u64) {
        let mut state = self.state.lock().unwrap();
        let timestamp = super::now_unix();
        state.price_updates += 1;
        state.prices_updated += count;
        state.prices_last_updated_at = Some(timestamp);
    }

    pub fn record_fiat_rates_update(&self, count: u64) {
        let mut state = self.state.lock().unwrap();
        let timestamp = super::now_unix();
        state.fiat_rates_updated += count;
        state.fiat_rates_last_updated_at = Some(timestamp);
    }
}

impl MetricsProvider for PriceMetrics {
    fn register(&self, registry: &mut Registry) {
        let state = self.state.lock().unwrap();

        let price_updates = Gauge::<i64, AtomicI64>::default();
        price_updates.set(state.price_updates as i64);

        let prices_updated = Gauge::<i64, AtomicI64>::default();
        prices_updated.set(state.prices_updated as i64);

        let prices_last_updated_at = Gauge::<i64, AtomicI64>::default();
        if let Some(ts) = state.prices_last_updated_at {
            prices_last_updated_at.set(ts as i64);
        }

        let fiat_rates_updated = Gauge::<i64, AtomicI64>::default();
        fiat_rates_updated.set(state.fiat_rates_updated as i64);

        let fiat_rates_last_updated_at = Gauge::<i64, AtomicI64>::default();
        if let Some(ts) = state.fiat_rates_last_updated_at {
            fiat_rates_last_updated_at.set(ts as i64);
        }

        registry.register("price_updates", "Total price update operations", price_updates);
        registry.register("prices_updated_count", "Total individual prices updated", prices_updated);
        registry.register("prices_last_updated_at", "Last price update timestamp", prices_last_updated_at);
        registry.register("fiat_rates_updated_count", "Total fiat rates updated", fiat_rates_updated);
        registry.register("fiat_rates_last_updated_at", "Last fiat rates update timestamp", fiat_rates_last_updated_at);
    }
}
