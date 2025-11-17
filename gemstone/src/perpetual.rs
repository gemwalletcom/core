use gem_hypercore::perpetual_formatter::PerpetualFormatter;
use primitives::PerpetualProvider;

#[derive(Debug, Default, uniffi::Object)]
pub struct Perpetual;

#[uniffi::export]
impl Perpetual {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn minimum_order_usd_amount(&self, provider: PerpetualProvider, price: f64, decimals: i32, leverage: u8) -> u64 {
        match provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::minimum_order_usd_amount(price, decimals, leverage),
        }
    }

    pub fn format_price(&self, provider: PerpetualProvider, price: f64, decimals: i32) -> String {
        match provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_price(price, decimals),
        }
    }

    pub fn format_size(&self, provider: PerpetualProvider, size: f64, decimals: i32) -> String {
        match provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_size(size, decimals),
        }
    }
}
