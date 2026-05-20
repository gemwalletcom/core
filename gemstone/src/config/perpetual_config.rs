pub const DEFAULT_LEVERAGE: u8 = 5;
pub const LEVERAGE_OPTIONS: &[u8] = &[1, 2, 3, 5, 10, 20, 25, 30, 40, 50];

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct PerpetualConfig {
    pub default_leverage: u8,
    pub leverage_options: Vec<u8>,
}

pub fn get_perpetual_config() -> PerpetualConfig {
    PerpetualConfig {
        default_leverage: DEFAULT_LEVERAGE,
        leverage_options: LEVERAGE_OPTIONS.to_vec(),
    }
}

pub fn select_leverage(desired: u8, options: &[u8]) -> u8 {
    options
        .iter()
        .copied()
        .filter(|&value| value <= desired)
        .max()
        .or_else(|| options.iter().copied().min())
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

        let constrained: &[u8] = &[1, 2, 3];
        assert_eq!(select_leverage(10, constrained), 3);

        let empty: &[u8] = &[];
        assert_eq!(select_leverage(5, empty), DEFAULT_LEVERAGE);
    }

    #[test]
    fn test_get_perpetual_config() {
        let config = get_perpetual_config();
        assert_eq!(config.default_leverage, DEFAULT_LEVERAGE);
        assert_eq!(config.leverage_options, LEVERAGE_OPTIONS);
    }
}
