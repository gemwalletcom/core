use number_formatter::price_suggestion;

#[derive(Default, uniffi::Object)]
pub struct PriceAlertFormatter {}

#[uniffi::export]
impl PriceAlertFormatter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn percentage_suggestions(&self, price: f64) -> Vec<i32> {
        price_suggestion::percentage_suggestions(price)
    }

    pub fn rounded_values(&self, price: f64, by_percent: f64) -> Vec<f64> {
        price_suggestion::price_rounded_values(price, by_percent)
    }
}
