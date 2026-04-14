pub fn percentage_suggestions(price: f64) -> Vec<i32> {
    let base = match price {
        p if p < 100.0 => 5,
        p if p < 10_000.0 => 3,
        _ => 2,
    };
    vec![base, base * 2, base * 3]
}

pub fn price_rounded_values(price: f64, by_percent: f64) -> Vec<f64> {
    if price < 0.01 || by_percent <= 0.0 {
        return vec![];
    }

    let lower_target = price * (1.0 - by_percent / 100.0);
    let upper_target = price * (1.0 + by_percent / 100.0);
    let step = price_step(lower_target);

    let lower = (lower_target / step).floor() * step;
    let upper = if step > 1.0 {
        (upper_target / step).round() * step
    } else {
        (upper_target / step).ceil() * step
    };

    vec![lower, upper]
}

fn price_step(value: f64) -> f64 {
    match value {
        v if v < 1.0 => 0.01,
        v if v < 100.0 => 1.0,
        v if v < 500.0 => 10.0,
        v if v < 10_000.0 => 50.0,
        _ => 1000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rounded(price: f64, by_percent: f64, expected: [f64; 2]) {
        let result = price_rounded_values(price, by_percent);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_percentage_suggestions() {
        assert_eq!(percentage_suggestions(50.0), vec![5, 10, 15]);
        assert_eq!(percentage_suggestions(500.0), vec![3, 6, 9]);
        assert_eq!(percentage_suggestions(10_000.0), vec![2, 4, 6]);
    }

    #[test]
    fn test_price_rounded_values() {
        assert_rounded(0.2829, 5.0, [0.26, 0.30]);
        assert_rounded(767.55, 5.0, [700.0, 800.0]);
        assert_rounded(95_432.0, 5.0, [90_000.0, 100_000.0]);

        assert!(price_rounded_values(0.0, 5.0).is_empty());
        assert!(price_rounded_values(100.0, -5.0).is_empty());
    }
}
