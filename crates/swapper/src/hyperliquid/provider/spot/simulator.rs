use std::{cmp::Ordering, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use gem_hypercore::models::spot::OrderbookLevel;

use crate::SwapperError;

pub(super) fn simulate_sell(amount: &BigDecimal, bids: &[OrderbookLevel]) -> Result<(BigDecimal, BigDecimal), SwapperError> {
    let mut remaining = amount.clone();
    let mut quote_total = BigDecimal::zero();

    for level in bids {
        let level_size = parse_decimal(&level.sz)?;
        let price = parse_decimal(&level.px)?;

        if level_size <= BigDecimal::zero() {
            continue;
        }

        let trade_size = match remaining.cmp(&level_size) {
            Ordering::Greater => level_size.clone(),
            _ => remaining.clone(),
        };

        quote_total += &trade_size * &price;
        remaining -= &trade_size;

        if remaining <= BigDecimal::zero() {
            let avg_price = &quote_total / amount.clone();
            return Ok((quote_total, avg_price));
        }
    }

    Err(SwapperError::NoQuoteAvailable)
}

pub(super) fn simulate_buy(amount: &BigDecimal, asks: &[OrderbookLevel]) -> Result<(BigDecimal, BigDecimal), SwapperError> {
    let mut remaining_quote = amount.clone();
    let mut base_total = BigDecimal::zero();
    let mut quote_total = BigDecimal::zero();

    for level in asks {
        let level_size = parse_decimal(&level.sz)?;
        let price = parse_decimal(&level.px)?;

        if level_size <= BigDecimal::zero() || price <= BigDecimal::zero() {
            continue;
        }

        let level_quote = &level_size * &price;

        if remaining_quote > level_quote {
            base_total += &level_size;
            quote_total += &level_quote;
            remaining_quote -= level_quote;
        } else {
            let partial_base = &remaining_quote / &price;
            base_total += &partial_base;
            quote_total += &remaining_quote;
            remaining_quote = BigDecimal::zero();
            break;
        }
    }

    if remaining_quote > BigDecimal::zero() || base_total <= BigDecimal::zero() {
        return Err(SwapperError::NoQuoteAvailable);
    }

    let avg_price = &quote_total / base_total.clone();
    Ok((base_total, avg_price))
}

fn parse_decimal(value: &str) -> Result<BigDecimal, SwapperError> {
    BigDecimal::from_str(value).map_err(|_| SwapperError::ComputeQuoteError("failed to parse orderbook level".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    fn format_decimal(decimal: &BigDecimal) -> String {
        decimal.normalized().to_string()
    }

    fn level(px: &str, sz: &str) -> OrderbookLevel {
        OrderbookLevel {
            px: px.to_string(),
            sz: sz.to_string(),
        }
    }

    #[test]
    fn test_simulate_sell() {
        let amount = BigDecimal::from_str("7").unwrap();
        let bids = vec![level("2", "3"), level("1.5", "5")];
        let (quote_out, avg_price) = simulate_sell(&amount, &bids).unwrap();
        assert_eq!(format_decimal(&quote_out), "11.5");
        assert_eq!(format_decimal(&(avg_price.clone() * amount.clone())), format_decimal(&quote_out));
    }

    #[test]
    fn test_simulate_sell_insufficient_depth() {
        let amount = BigDecimal::from_str("10").unwrap();
        let bids = vec![level("2", "3"), level("1.5", "5")];
        assert!(matches!(simulate_sell(&amount, &bids), Err(SwapperError::NoQuoteAvailable)));
    }

    #[test]
    fn test_simulate_buy() {
        let amount = BigDecimal::from_str("10").unwrap();
        let asks = vec![level("2", "3"), level("3", "5")];
        let (base_out, avg_price) = simulate_buy(&amount, &asks).unwrap();
        assert_eq!(format_decimal(&(avg_price.clone() * base_out.clone())), format_decimal(&amount));
        assert!(base_out > BigDecimal::zero());
    }

    #[test]
    fn test_simulate_buy_insufficient_depth() {
        let amount = BigDecimal::from_str("20").unwrap();
        let asks = vec![level("2", "3"), level("3", "5")];
        assert!(matches!(simulate_buy(&amount, &asks), Err(SwapperError::NoQuoteAvailable)));
    }
}
