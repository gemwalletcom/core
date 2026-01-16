use std::str::FromStr;

use bigdecimal::{BigDecimal, Zero};
use gem_hypercore::models::spot::OrderbookLevel;

use crate::SwapperError;

#[derive(Debug, Clone)]
pub(super) struct SimulationResult {
    pub amount_out: BigDecimal,
    pub limit_price: BigDecimal,
}

pub(super) fn simulate_sell(amount: &BigDecimal, bids: &[OrderbookLevel]) -> Result<SimulationResult, SwapperError> {
    let mut remaining = amount.clone();
    let mut quote_total = BigDecimal::zero();
    let mut min_price: Option<BigDecimal> = None;

    for level in bids {
        let level_size = parse_decimal(&level.sz)?;
        let price = parse_decimal(&level.px)?;
        if level_size <= BigDecimal::zero() {
            continue;
        }

        let trade_size = remaining.clone().min(level_size);
        quote_total += &trade_size * &price;
        remaining -= &trade_size;
        min_price = Some(min_price.map_or(price.clone(), |p| p.min(price.clone())));

        if remaining <= BigDecimal::zero() {
            return Ok(SimulationResult {
                amount_out: quote_total,
                limit_price: min_price.unwrap(),
            });
        }
    }

    Err(SwapperError::NoQuoteAvailable)
}

pub(super) fn simulate_buy(amount: &BigDecimal, asks: &[OrderbookLevel]) -> Result<SimulationResult, SwapperError> {
    let mut remaining_quote = amount.clone();
    let mut base_total = BigDecimal::zero();
    let mut max_price: Option<BigDecimal> = None;

    for level in asks {
        let level_size = parse_decimal(&level.sz)?;
        let price = parse_decimal(&level.px)?;
        if level_size <= BigDecimal::zero() || price <= BigDecimal::zero() {
            continue;
        }

        let level_quote = &level_size * &price;
        if remaining_quote > level_quote {
            base_total += &level_size;
            remaining_quote -= level_quote;
            max_price = Some(max_price.map_or(price.clone(), |p| p.max(price.clone())));
        } else {
            base_total += &remaining_quote / &price;
            max_price = Some(max_price.map_or(price.clone(), |p| p.max(price.clone())));
            remaining_quote = BigDecimal::zero();
            break;
        }
    }

    if remaining_quote > BigDecimal::zero() || base_total <= BigDecimal::zero() {
        return Err(SwapperError::NoQuoteAvailable);
    }

    Ok(SimulationResult {
        amount_out: base_total,
        limit_price: max_price.unwrap(),
    })
}

fn parse_decimal(value: &str) -> Result<BigDecimal, SwapperError> {
    BigDecimal::from_str(value).map_err(|_| SwapperError::ComputeQuoteError("failed to parse orderbook level".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use number_formatter::BigNumberFormatter;
    use std::str::FromStr;

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
        let SimulationResult {
            amount_out: quote_out,
            limit_price: min_price,
        } = simulate_sell(&amount, &bids).unwrap();
        let expected = BigDecimal::from_str("12").unwrap();

        let quote_str = BigNumberFormatter::decimal_to_string(&quote_out, 6);
        let expected_str = BigNumberFormatter::decimal_to_string(&expected, 6);
        assert_eq!(quote_str, expected_str);

        let avg_total = quote_out.clone() / amount.clone() * amount;
        let avg_total_str = BigNumberFormatter::decimal_to_string(&avg_total, 6);
        assert_eq!(avg_total_str, expected_str);
        assert_eq!(min_price, BigDecimal::from_str("1.5").unwrap());
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
        let SimulationResult {
            amount_out: base_out,
            limit_price: max_price,
        } = simulate_buy(&amount, &asks).unwrap();
        let avg_price = &amount / &base_out;
        let product = avg_price * base_out.clone();
        let product_str = BigNumberFormatter::decimal_to_string(&product, 6);
        let amount_str = BigNumberFormatter::decimal_to_string(&amount, 6);
        assert_eq!(product_str, amount_str);
        assert!(base_out > BigDecimal::zero());
        assert_eq!(max_price, BigDecimal::from_str("3").unwrap());
    }

    #[test]
    fn test_simulate_buy_insufficient_depth() {
        let amount = BigDecimal::from_str("25").unwrap();
        let asks = vec![level("2", "3"), level("3", "5")];
        assert!(matches!(simulate_buy(&amount, &asks), Err(SwapperError::NoQuoteAvailable)));
    }
}
