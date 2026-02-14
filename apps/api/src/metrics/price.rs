use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicI64;

pub fn register(registry: &mut Registry) {
    let assets_total = Gauge::<i64, AtomicI64>::default();

    registry.register("price_assets_total", "Total number of tracked price assets", assets_total);
}
