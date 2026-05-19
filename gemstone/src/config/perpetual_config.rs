pub const DEFAULT_LEVERAGE: i32 = 5;
pub const LEVERAGE_OPTIONS: &[i32] = &[1, 2, 3, 5, 10, 20, 25, 30, 40, 50];

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct PerpetualConfig {
    pub default_leverage: i32,
    pub leverage_options: Vec<i32>,
}

pub fn get_perpetual_config() -> PerpetualConfig {
    PerpetualConfig {
        default_leverage: DEFAULT_LEVERAGE,
        leverage_options: LEVERAGE_OPTIONS.to_vec(),
    }
}

pub fn select_leverage(desired: i32, from: &[i32]) -> i32 {
    from.iter()
        .copied()
        .filter(|&value| value <= desired)
        .max()
        .or_else(|| from.iter().copied().min())
        .unwrap_or(DEFAULT_LEVERAGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_leverage() {
        assert_eq!(select_leverage(0, LEVERAGE_OPTIONS), 1);
        assert_eq!(select_leverage(4, LEVERAGE_OPTIONS), 3);
        assert_eq!(select_leverage(5, LEVERAGE_OPTIONS), 5);
        assert_eq!(select_leverage(7, LEVERAGE_OPTIONS), 5);
        assert_eq!(select_leverage(50, LEVERAGE_OPTIONS), 50);
        assert_eq!(select_leverage(100, LEVERAGE_OPTIONS), 50);

        let constrained: &[i32] = &[1, 2, 3];
        assert_eq!(select_leverage(10, constrained), 3);

        let empty: &[i32] = &[];
        assert_eq!(select_leverage(5, empty), DEFAULT_LEVERAGE);
    }

    #[test]
    fn test_get_perpetual_config() {
        let config = get_perpetual_config();
        assert_eq!(config.default_leverage, DEFAULT_LEVERAGE);
        assert_eq!(config.leverage_options, LEVERAGE_OPTIONS);
    }
}
