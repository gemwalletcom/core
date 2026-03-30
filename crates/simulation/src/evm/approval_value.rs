use num_bigint::BigUint;
use num_traits::One;
use primitives::SimulationHeader;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ApprovalValue {
    Exact(BigUint),
    Unlimited,
}

impl ApprovalValue {
    pub(crate) fn from_raw(raw_value: &str) -> Option<Self> {
        let Ok(value) = raw_value.parse::<BigUint>() else {
            return None;
        };

        if Self::is_unlimited(&value) {
            return Some(Self::Unlimited);
        }
        Some(Self::Exact(value))
    }

    fn is_unlimited(value: &BigUint) -> bool {
        Self::is_max_unsigned(value, 160) || Self::is_max_unsigned(value, 256)
    }

    fn is_max_unsigned(value: &BigUint, bit_width: u32) -> bool {
        value == &((BigUint::one() << bit_width) - BigUint::one())
    }

    pub(crate) fn display_value(&self) -> String {
        match self {
            Self::Exact(value) => value.to_string(),
            Self::Unlimited => "Unlimited".to_string(),
        }
    }

    pub(crate) fn to_simulation_header(&self, asset_id: primitives::AssetId) -> SimulationHeader {
        match self {
            Self::Exact(value) => SimulationHeader {
                asset_id,
                value: value.to_string(),
                is_unlimited: false,
            },
            Self::Unlimited => SimulationHeader {
                asset_id,
                value: String::new(),
                is_unlimited: true,
            },
        }
    }
}
