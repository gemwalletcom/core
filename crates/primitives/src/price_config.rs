use std::time::Duration;

#[derive(Clone, Copy)]
pub struct PriceConfig {
    pub primary_price_max_age: Duration,
}
