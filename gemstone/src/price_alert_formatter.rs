use number_formatter::price_suggestion;

#[uniffi::export]
pub fn price_alert_percentage_suggestions(price: f64) -> Vec<i32> {
    price_suggestion::percentage_suggestions(price)
}

#[uniffi::export]
pub fn price_alert_rounded_values(price: f64, by_percent: f64) -> Vec<f64> {
    price_suggestion::price_rounded_values(price, by_percent)
}
