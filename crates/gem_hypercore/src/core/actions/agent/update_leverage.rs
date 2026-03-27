use primitives::PerpetualMarginType;
use serde::{Deserialize, Serialize};

// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.
#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateLeverage {
    pub r#type: String,
    pub asset: u32,
    #[serde(rename = "isCross")]
    pub is_cross: bool,
    pub leverage: u8,
}

impl UpdateLeverage {
    pub fn new(asset: u32, is_cross: bool, leverage: u8) -> Self {
        Self {
            r#type: "updateLeverage".to_string(),
            asset,
            is_cross,
            leverage,
        }
    }

    pub fn from_margin_type(asset: u32, margin_type: &PerpetualMarginType, leverage: u8) -> Self {
        Self::new(asset, *margin_type == PerpetualMarginType::Cross, leverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_margin_type_cross() {
        let leverage = UpdateLeverage::from_margin_type(1, &PerpetualMarginType::Cross, 10);
        assert!(leverage.is_cross);
    }

    #[test]
    fn test_from_margin_type_isolated() {
        let leverage = UpdateLeverage::from_margin_type(1, &PerpetualMarginType::Isolated, 10);
        assert!(!leverage.is_cross);
    }
}
