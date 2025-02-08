use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use std::str::FromStr;

pub struct TonNanoConverter;

impl TonNanoConverter {
    pub fn to_nano(amount: f64) -> Option<BigInt> {
        if !amount.is_finite() {
            return None;
        }

        let amount_str = if amount.log10() <= 6.0 {
            format!("{:.9}", amount)
        } else if amount.fract() == 0.0 {
            format!("{:.0}", amount)
        } else {
            return None;
        };

        let mut neg = false;
        let mut src = amount_str;
        if src.starts_with('-') {
            neg = true;
            src = src.trim_start_matches('-').to_string();
        }

        if src == "." {
            return None;
        }

        let parts: Vec<&str> = src.split('.').collect();
        if parts.len() > 2 {
            return None;
        }

        let whole = parts.get(0).unwrap_or(&"0");
        let mut frac = parts.get(1).unwrap_or(&"0").to_string();
        while frac.len() < 9 {
            frac.push('0');
        }
        if frac.len() > 9 {
            return None;
        }

        let whole_part = BigInt::from_str(whole).ok()?;
        let frac_part = BigInt::from_str(&frac).ok()?;
        let nano_value = whole_part * BigInt::from(1_000_000_000) + frac_part;

        Some(if neg { -nano_value } else { nano_value })
    }

    pub fn from_nano(amount: BigInt) -> Option<f64> {
        let neg = amount < BigInt::zero();
        let abs_amount = if neg { -amount.clone() } else { amount.clone() };

        let whole = &abs_amount / BigInt::from(1_000_000_000);
        let frac = &abs_amount % BigInt::from(1_000_000_000);

        let mut frac_str = frac.to_str_radix(10);
        while frac_str.len() < 9 {
            frac_str.insert(0, '0');
        }

        let frac_str = frac_str.trim_end_matches('0');
        let result_str = if frac_str.is_empty() {
            format!("{}", whole)
        } else {
            format!("{}.{}", whole, frac_str)
        };

        let result = f64::from_str(&result_str).ok()?;
        Some(if neg { -result } else { result })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_nano() {
        assert_eq!(TonNanoConverter::to_nano(1.5), Some(BigInt::from(1_500_000_000)));
        assert_eq!(TonNanoConverter::to_nano(0.000000001), Some(BigInt::from(1)));
        assert_eq!(TonNanoConverter::to_nano(0.4), Some(BigInt::from(400_000_000)));
    }

    #[test]
    fn test_from_nano() {
        assert_eq!(TonNanoConverter::from_nano(BigInt::from(1_500_000_000)), Some(1.5));
        assert_eq!(TonNanoConverter::from_nano(BigInt::from(1)), Some(0.000000001));
        assert_eq!(TonNanoConverter::from_nano(BigInt::from(400_000_000)), Some(0.4));
    }
}
