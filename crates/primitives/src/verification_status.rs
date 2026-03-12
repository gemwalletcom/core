use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Suspicious,
}

impl VerificationStatus {
    pub fn from_verified(is_verified: bool) -> Self {
        if is_verified { Self::Verified } else { Self::Unverified }
    }

    pub fn is_verified(self) -> bool {
        match self {
            Self::Verified => true,
            Self::Unverified | Self::Suspicious => false,
        }
    }
}
