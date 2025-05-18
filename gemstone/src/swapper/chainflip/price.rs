use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_bigint::ToBigInt;
use num_traits::FromPrimitive;

pub fn apply_slippage(original_price: f64, slippage_bps: u32) -> f64 {
    original_price * (1.0 - slippage_bps as f64 / 10000.0)
}
/// https://docs.chainflip.io/lp/integrations/lp-api#hex-price
pub fn price_to_hex_price(price: f64, quote_asset_decimals: u32, base_asset_decimals: u32) -> Result<String, String> {
    if price.is_nan() || price.is_infinite() {
        return Err(format!("Input price ({}) is NaN or Infinity.", price));
    }
    let price = BigDecimal::from_f64(price).ok_or("Failed to convert price to BigDecimal")?;
    let shifted = price * BigInt::from(2).pow(128);
    let hex_price = shifted * BigInt::from(10).pow(quote_asset_decimals) / BigDecimal::from_bigint(BigInt::from(10).pow(base_asset_decimals), 0);
    let hex_price_str = hex_price.to_bigint().map(|x| x.to_str_radix(16)).ok_or("Failed to convert to BigInt")?;
    let padded = format!("0x{}{}", if hex_price_str.len() % 2 == 0 { "" } else { "0" }, hex_price_str);
    Ok(padded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_slippage_sell() {
        assert_eq!(apply_slippage(100.0, 100), 99.0);
    }

    #[test]
    fn test_example_10000_usdc_eth() {
        // 10000 USDC/ETH, base asset is USDC, quote asset is ETH
        assert_eq!(price_to_hex_price(10000.0, 6, 18).unwrap(), "0x2af31dc4611873bf3f70834acd");
    }
}
