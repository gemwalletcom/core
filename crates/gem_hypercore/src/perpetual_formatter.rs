// https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/tick-and-lot-size
// https://hyperliquid.gitbook.io/hyperliquid-docs/trading/contract-specifications

const MIN_ORDER_VALUE_USD: f64 = 10.0;
const USDC_CENTS_MULTIPLIER: f64 = 100.0;
const USDC_DECIMALS_MULTIPLIER: f64 = 1_000_000.0;

pub struct PerpetualFormatter;

impl PerpetualFormatter {
    /// Hyperliquid requires minimum $10 notional value (size × price).
    pub fn minimum_order_usd_amount(price: f64, sz_decimals: i32, leverage: u8) -> u64 {
        let size_multiplier = 10_f64.powi(sz_decimals);
        let rounded_size = ((MIN_ORDER_VALUE_USD / price) * size_multiplier).ceil() / size_multiplier;
        let min_usd = ((rounded_size * price / leverage as f64) * USDC_CENTS_MULTIPLIER).ceil() / USDC_CENTS_MULTIPLIER;

        (min_usd * USDC_DECIMALS_MULTIPLIER) as u64
    }

    pub fn format_price(price: f64, sz_decimals: i32) -> String {
        if price == 0.0 {
            return "0".to_string();
        }

        let max_decimals = (6 - sz_decimals).max(0);
        let magnitude = price.abs().log10().floor();
        let sig_fig_decimals = (4.0 - magnitude).max(0.0) as i32;
        let decimals = sig_fig_decimals.min(max_decimals) as usize;

        format_and_trim(price, decimals)
    }

    pub fn format_size(size: f64, sz_decimals: i32) -> String {
        let decimals = sz_decimals.max(0) as usize;
        let multiplier = 10_f64.powi(sz_decimals);
        let value = (size * multiplier + 0.5).floor() / multiplier;

        format_and_trim(value, decimals)
    }
}

fn format_and_trim(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.decimals$}", value, decimals = decimals);

    if formatted.contains('.') {
        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_order_usd_amount() {
        assert_eq!(PerpetualFormatter::minimum_order_usd_amount(100_000.0, 5, 1), 10_000_000);
        assert_eq!(PerpetualFormatter::minimum_order_usd_amount(3_500.0, 4, 3), 3_390_000);
        assert_eq!(PerpetualFormatter::minimum_order_usd_amount(487.0, 2, 1), 14_610_000);
        assert_eq!(PerpetualFormatter::minimum_order_usd_amount(200.0, 1, 10), 2_000_000);
        assert_eq!(PerpetualFormatter::minimum_order_usd_amount(0.5, 0, 1), 10_000_000);
    }

    #[test]
    fn test_format_price() {
        assert_eq!(PerpetualFormatter::format_price(0.002877, 0), "0.002877");
        assert_eq!(PerpetualFormatter::format_price(0.00284, 0), "0.00284");
        assert_eq!(PerpetualFormatter::format_price(0.003003, 0), "0.003003");
        assert_eq!(PerpetualFormatter::format_price(12345.678, 0), "12346");
        assert_eq!(PerpetualFormatter::format_price(1234.5, 0), "1234.5");
        assert_eq!(PerpetualFormatter::format_price(123.45, 0), "123.45");
        assert_eq!(PerpetualFormatter::format_price(3397.10, 0), "3397.1");
        assert_eq!(PerpetualFormatter::format_price(3532.984, 0), "3533");
        assert_eq!(PerpetualFormatter::format_price(3261.216, 0), "3261.2");
        assert_eq!(PerpetualFormatter::format_price(99.999, 0), "99.999");
        assert_eq!(PerpetualFormatter::format_price(0.005849, 0), "0.005849");
        assert_eq!(PerpetualFormatter::format_price(0.0061415, 0), "0.006142");
        assert_eq!(PerpetualFormatter::format_price(0.0052641, 0), "0.005264");

        assert_eq!(PerpetualFormatter::format_price(1234.567, 1), "1234.6");
        assert_eq!(PerpetualFormatter::format_price(123.456, 1), "123.46");
        assert_eq!(PerpetualFormatter::format_price(0.0012345, 1), "0.00123");

        assert_eq!(PerpetualFormatter::format_price(123.456, 2), "123.46");
        assert_eq!(PerpetualFormatter::format_price(12.3456, 2), "12.346");
        assert_eq!(PerpetualFormatter::format_price(1.23456, 2), "1.2346");

        assert_eq!(PerpetualFormatter::format_price(3397.10, 4), "3397.1");
        assert_eq!(PerpetualFormatter::format_price(3532.984, 4), "3533");
        assert_eq!(PerpetualFormatter::format_price(0.005849, 4), "0.01");

        assert_eq!(PerpetualFormatter::format_price(0.0, 6), "0");
        assert_eq!(PerpetualFormatter::format_price(1.0, 6), "1");
        assert_eq!(PerpetualFormatter::format_price(0.000001, 6), "0");
        assert_eq!(PerpetualFormatter::format_price(123.456, 6), "123");

        assert_eq!(PerpetualFormatter::format_price(-123.456, 0), "-123.46");
        assert_eq!(PerpetualFormatter::format_price(0.0000001, 0), "0");
        assert_eq!(PerpetualFormatter::format_price(999999.0, 0), "999999");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(PerpetualFormatter::format_size(123.456789, 0), "123");
        assert_eq!(PerpetualFormatter::format_size(0.123456, 0), "0");
        assert_eq!(PerpetualFormatter::format_size(1000.5, 0), "1001");
        assert_eq!(PerpetualFormatter::format_size(0.9, 0), "1");
        assert_eq!(PerpetualFormatter::format_size(0.4, 0), "0");

        assert_eq!(PerpetualFormatter::format_size(123.456, 1), "123.5");
        assert_eq!(PerpetualFormatter::format_size(0.123456, 1), "0.1");
        assert_eq!(PerpetualFormatter::format_size(1.05, 1), "1.1");

        assert_eq!(PerpetualFormatter::format_size(0.123456, 3), "0.123");
        assert_eq!(PerpetualFormatter::format_size(1.234567, 3), "1.235");

        assert_eq!(PerpetualFormatter::format_size(0.123456789, 6), "0.123457");

        assert_eq!(PerpetualFormatter::format_size(-123.456, 2), "-123.46");
    }

    #[test]
    fn test_format_and_trim() {
        assert_eq!(format_and_trim(123.456, 2), "123.46");
        assert_eq!(format_and_trim(123.0, 2), "123");
        assert_eq!(format_and_trim(123.400, 3), "123.4");
        assert_eq!(format_and_trim(0.0, 2), "0");
        assert_eq!(format_and_trim(1.0000, 4), "1");
        assert_eq!(format_and_trim(1.2000, 4), "1.2");
        assert_eq!(format_and_trim(123.0, 0), "123");
        assert_eq!(format_and_trim(-123.450, 2), "-123.45");
    }
}
