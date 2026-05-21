use crate::{
    SwapperError,
    mayan::{model::QuoteType, wormhole_chain::WormholeChain},
};
use number_formatter::BigNumberFormatter;
use serde_json::Value;
use std::str::FromStr;

pub(super) fn fractional_amount<T>(amount: &Value, decimals: u32) -> Result<T, SwapperError>
where
    T: FromStr,
    SwapperError: From<T::Err>,
{
    Ok(fractional_amount_value(amount, decimals)?.parse::<T>()?)
}

pub(super) fn min_amount_out(amount: &Value, token_decimals: u32, chain: &str, quote_type: &QuoteType) -> Result<u64, SwapperError> {
    fractional_amount(amount, token_decimals.min(amount_decimals_cap(chain, quote_type)?))
}

pub(super) fn gas_drop_amount(amount: &Value, chain: &str, quote_type: &QuoteType, hypercore_deposit: bool) -> Result<u64, SwapperError> {
    if hypercore_deposit {
        return Ok(0);
    }
    fractional_amount(amount, gas_decimals(chain)?.min(amount_decimals_cap(chain, quote_type)?))
}

pub(super) fn optional_bps_u8(value: Option<u32>) -> Result<u8, SwapperError> {
    let Some(value) = value else {
        return Ok(0);
    };
    value.try_into().map_err(|_| SwapperError::InvalidRoute)
}

pub(super) fn fractional_amount_value(amount: &Value, decimals: u32) -> Result<String, SwapperError> {
    let amount = value_to_query(amount)?;
    BigNumberFormatter::value_from_amount_truncated(&amount, decimals).map_err(|_| SwapperError::InvalidRoute)
}

pub(super) fn value_to_query(amount: &Value) -> Result<String, SwapperError> {
    match amount {
        Value::Number(number) => Ok(number.to_string()),
        Value::String(value) => Ok(value.clone()),
        _ => Err(SwapperError::InvalidRoute),
    }
}

pub(super) fn gas_decimals(chain: &str) -> Result<u32, SwapperError> {
    match WormholeChain::from_name(chain)? {
        WormholeChain::Solana | WormholeChain::Sui | WormholeChain::Ton => Ok(9),
        _ => Ok(18),
    }
}

fn amount_decimals_cap(chain: &str, quote_type: &QuoteType) -> Result<u32, SwapperError> {
    match WormholeChain::from_name(chain)? {
        WormholeChain::Ton if quote_type != &QuoteType::Swift => Err(SwapperError::InvalidRoute),
        WormholeChain::Ton => Ok(u32::MAX),
        _ => Ok(8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fractional_amount_value() {
        assert_eq!(fractional_amount::<u64>(&serde_json::json!(1.183818719), 9).unwrap(), 1_183_818_719);
        assert_eq!(
            fractional_amount_value(&serde_json::json!("123456789012345678.123456789"), 18).unwrap(),
            "123456789012345678123456789000000000"
        );
        assert_eq!(fractional_amount::<u64>(&serde_json::json!("0.000000009"), 9).unwrap(), 9);
        assert_eq!(fractional_amount_value(&serde_json::json!("-1"), 9), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_protocol_amounts_keep_eight_decimal_cap() {
        let amount = serde_json::json!(1.183818719);
        assert_eq!(min_amount_out(&amount, 9, WormholeChain::Solana.name(), &QuoteType::Swift).unwrap(), 118_381_871);
        assert_eq!(min_amount_out(&amount, 9, WormholeChain::Ton.name(), &QuoteType::Swift).unwrap(), 1_183_818_719);
    }
}
