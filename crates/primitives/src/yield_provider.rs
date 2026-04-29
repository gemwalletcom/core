use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::{Chain, DelegationValidator, StakeProviderType};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum YieldProvider {
    Yo,
    Tonstakers,
}

impl YieldProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::Yo => "Yo",
            Self::Tonstakers => "Tonstakers",
        }
    }

    pub fn delegation_validator(&self, chain: Chain) -> DelegationValidator {
        DelegationValidator {
            chain,
            id: self.as_ref().to_string(),
            name: self.name().to_string(),
            is_active: true,
            commission: 0.0,
            apr: 0.0,
            provider_type: StakeProviderType::Earn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_validator() {
        let result = YieldProvider::Yo.delegation_validator(Chain::Base);

        assert_eq!(result.id, "yo");
        assert_eq!(result.name, "Yo");
        assert_eq!(result.chain, Chain::Base);
        assert_eq!(result.apr, 0.0);
        assert_eq!(result.provider_type, StakeProviderType::Earn);
    }
}
