use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID, swap::SwapData};
use std::error::Error;

use crate::perpetual_formatter::USDC_DECIMALS_MULTIPLIER;

const HYPERCORE_USER_FEE_RATE_SCALE: f64 = 1_000_000.0;
const HYPERCORE_TRADE_FEE_MULTIPLIER: f64 = 20.0;
const HYPERCORE_BUILDER_FEE_RATE_SCALE: f64 = 100_000.0;
const HYPERCORE_PERPETUAL_USDC_DECIMALS: i32 = 6;

pub fn calculate_perpetual_fee_amount(fiat_value: f64, fee_rate: i64) -> BigInt {
    let fee_rate_f64 = fee_rate as f64;
    let result = fiat_value * (fee_rate_f64 / HYPERCORE_USER_FEE_RATE_SCALE) * HYPERCORE_TRADE_FEE_MULTIPLIER * USDC_DECIMALS_MULTIPLIER;
    BigInt::from(result as i64)
}

pub fn calculate_spot_fee_amount(swap_data: &SwapData, from_asset: &Asset, to_asset: &Asset, fee_rate: i64, builder_fee_bps: u32) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let fiat_value = calculate_spot_usdc_value(swap_data, from_asset, to_asset, builder_fee_bps)?;
    let usdc_decimals = spot_usdc_decimals(from_asset, to_asset)?;
    let trade_fee = calculate_perpetual_fee_amount(fiat_value * decimal_scale(usdc_decimals - HYPERCORE_PERPETUAL_USDC_DECIMALS), fee_rate);
    let builder_fee = BigInt::from((fiat_value * f64::from(builder_fee_bps) * decimal_scale(usdc_decimals - 5)) as i64);

    Ok(trade_fee + builder_fee)
}

fn calculate_spot_usdc_value(swap_data: &SwapData, from_asset: &Asset, to_asset: &Asset, builder_fee_bps: u32) -> Result<f64, Box<dyn Error + Send + Sync>> {
    let usdc_from = from_asset.id == *HYPERCORE_SPOT_USDC_ASSET_ID;
    let usdc_to = to_asset.id == *HYPERCORE_SPOT_USDC_ASSET_ID;

    match (usdc_from, usdc_to) {
        (true, false) => quote_value(&swap_data.quote.from_value, from_asset.decimals),
        (false, true) => {
            let net_output = quote_value(&swap_data.quote.to_value, to_asset.decimals)?;
            let fee_factor = 1.0 - f64::from(builder_fee_bps) / HYPERCORE_BUILDER_FEE_RATE_SCALE;
            Ok(net_output / fee_factor)
        }
        _ => Err("spot swap quote must have exactly one USDC leg".into()),
    }
}

fn quote_value(value: &str, decimals: i32) -> Result<f64, Box<dyn Error + Send + Sync>> {
    Ok(BigNumberFormatter::value(value, decimals)?.parse::<f64>()?)
}

fn spot_usdc_decimals(from_asset: &Asset, to_asset: &Asset) -> Result<i32, Box<dyn Error + Send + Sync>> {
    if from_asset.id == *HYPERCORE_SPOT_USDC_ASSET_ID {
        return Ok(from_asset.decimals);
    }
    if to_asset.id == *HYPERCORE_SPOT_USDC_ASSET_ID {
        return Ok(to_asset.decimals);
    }
    Err("spot swap quote must have exactly one USDC leg".into())
}

fn decimal_scale(power: i32) -> f64 {
    10_i64.pow(power as u32) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        SwapProvider,
        known_assets::{HYPERCORE_SPOT_HYPE, HYPERCORE_SPOT_USDC},
    };

    #[test]
    fn calculate_perpetual_fee_amount_cases() {
        for (fiat_value, fee_rate, expected) in [
            (100.0, 50000, 100_000_000_i64),
            (1000.0, 0, 0),
            (0.0, 25000, 0),
            (1.0, 1000, 20_000),
            (10000.0, 100000, 20_000_000_000),
            (500.0, 30000, 300_000_000),
            (150.0, 43, 129_000),
            (1000.0, 43, 860_000),
        ] {
            assert_eq!(calculate_perpetual_fee_amount(fiat_value, fee_rate), BigInt::from(expected));
        }
    }

    #[test]
    fn calculate_spot_fee_amount_cases() {
        for (swap_data, from_asset, to_asset, expected) in [
            (
                SwapData::mock_with_values(SwapProvider::Hyperliquid, "30000000", "1181917897"),
                &HYPERCORE_SPOT_HYPE,
                &HYPERCORE_SPOT_USDC,
                1_856_445_u64,
            ),
            (
                SwapData::mock_with_values(SwapProvider::Hyperliquid, "1197900000", "29986500"),
                &HYPERCORE_SPOT_USDC,
                &HYPERCORE_SPOT_HYPE,
                1_880_702_u64,
            ),
        ] {
            let result = calculate_spot_fee_amount(&swap_data, from_asset, to_asset, 56, 45).unwrap();
            assert_eq!(result, BigInt::from(expected));
        }
    }
}
