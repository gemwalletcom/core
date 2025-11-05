use std::str::FromStr;

pub(super) use super::super::math::scale_units;
use crate::SwapperError;
use bigdecimal::{BigDecimal, Zero};
use num_traits::ToPrimitive;
use number_formatter::BigNumberFormatter;

pub(super) const SPOT_ASSET_OFFSET: u32 = 10_000;
const MAX_DECIMAL_SCALE: u32 = 6;

#[derive(Debug, Clone, Copy)]
pub(super) enum SpotSide {
    Buy,
    Sell,
}

impl SpotSide {
    pub(super) fn is_buy(self) -> bool {
        matches!(self, SpotSide::Buy)
    }
}

pub(super) fn format_decimal(value: &BigDecimal) -> String {
    format_decimal_with_scale(value, MAX_DECIMAL_SCALE)
}

pub(super) fn format_decimal_with_scale(value: &BigDecimal, scale: u32) -> String {
    BigNumberFormatter::decimal_to_string(value, scale)
}

pub(super) fn format_order_size(amount: &BigDecimal, decimals: u32) -> Result<String, SwapperError> {
    let value = amount
        .to_f64()
        .ok_or_else(|| SwapperError::InvalidAmount("failed to convert amount".to_string()))?;
    let rounded = round_to_decimals(value, decimals);
    let formatted = if decimals == 0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.decimals$}", decimals = decimals as usize)
    };
    let big_decimal = BigDecimal::from_str(&formatted).map_err(|_| SwapperError::InvalidAmount("failed to format size".to_string()))?;
    Ok(BigNumberFormatter::decimal_to_string(&big_decimal, decimals))
}

pub(super) fn spot_asset_index(market_index: u32) -> u32 {
    SPOT_ASSET_OFFSET + market_index
}

pub(super) fn apply_slippage(limit_price: &BigDecimal, side: SpotSide, slippage_bps: u32, price_decimals: u32) -> Result<BigDecimal, SwapperError> {
    if limit_price <= &BigDecimal::zero() {
        return Err(SwapperError::InvalidAmount("invalid limit price".to_string()));
    }

    let limit_price_f64 = limit_price
        .to_f64()
        .ok_or_else(|| SwapperError::InvalidAmount("failed to convert price".to_string()))?;

    let slippage_fraction = slippage_bps as f64 / 10_000.0;
    let multiplier = if side.is_buy() { 1.0 + slippage_fraction } else { 1.0 - slippage_fraction };

    if multiplier <= 0.0 {
        return Err(SwapperError::InvalidAmount("slippage multiplier not positive".to_string()));
    }

    let adjusted = limit_price_f64 * multiplier;
    let rounded = round_to_significant_and_decimal(adjusted, 5, price_decimals);

    let formatted = if price_decimals == 0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.price_decimals$}", price_decimals = price_decimals as usize)
    };

    BigDecimal::from_str(&formatted).map_err(|_| SwapperError::InvalidAmount("failed to format limit price".to_string()))
}

fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

fn round_to_significant_and_decimal(value: f64, sig_figs: u32, max_decimals: u32) -> f64 {
    if value == 0.0 {
        return 0.0;
    }

    let abs_value = value.abs();
    let magnitude = abs_value.log10().floor() as i32;
    let scale = 10f64.powi(sig_figs as i32 - magnitude - 1);
    let rounded = (abs_value * scale).round() / scale;
    round_to_decimals(rounded.copysign(value), max_decimals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use number_formatter::BigNumberFormatter;
    use std::str::FromStr;

    #[test]
    fn test_format_order_size_rounds() {
        let value = BigDecimal::from_str("0.131").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "0.13");

        let value = BigDecimal::from_str("0.189834").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "0.19");

        let value = BigDecimal::from_str("0.10").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "0.1");

        let value = BigDecimal::from_str("-0.131").unwrap();
        assert_eq!(format_order_size(&value, 2).unwrap(), "-0.13");
    }

    #[test]
    fn test_spot_asset_index_offset() {
        assert_eq!(spot_asset_index(0), SPOT_ASSET_OFFSET);
        assert_eq!(spot_asset_index(107), SPOT_ASSET_OFFSET + 107);
    }

    #[test]
    fn test_apply_slippage_buy_increases_price() {
        let price = BigDecimal::from_str("100").unwrap();
        let adjusted = apply_slippage(&price, SpotSide::Buy, 1000, 2).unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&adjusted, 2), "110");
    }

    #[test]
    fn test_apply_slippage_sell_decreases_price() {
        let price = BigDecimal::from_str("100").unwrap();
        let adjusted = apply_slippage(&price, SpotSide::Sell, 500, 2).unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&adjusted, 2), "95");
    }

    #[test]
    fn test_apply_slippage_zero_returns_same_price() {
        let price = BigDecimal::from_str("42.123456").unwrap();
        let adjusted = apply_slippage(&price, SpotSide::Sell, 0, 4).unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&adjusted, 4), "42.123");
    }

    #[test]
    fn test_apply_slippage_invalid_when_multiplier_non_positive() {
        let price = BigDecimal::from_str("10").unwrap();
        assert!(apply_slippage(&price, SpotSide::Sell, 10001, 2).is_err());
    }
}
