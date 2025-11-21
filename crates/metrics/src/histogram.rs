use prometheus_client::metrics::histogram::Histogram;

pub const LATENCY_BUCKETS: [f64; 5] = [0.1, 0.5, 1.0, 2.5, 5.0];

pub fn latency() -> Histogram {
    Histogram::new(LATENCY_BUCKETS.into_iter())
}
