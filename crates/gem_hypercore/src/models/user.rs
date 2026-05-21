use serde::{Deserialize, Serialize};
use serde_serializers::f64::deserialize_f64_from_str;

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRole {
    pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserAbstractionMode {
    Default,
    Disabled,
    DexAbstraction,
    UnifiedAccount,
    PortfolioMargin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub address: String,
    pub valid_until: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFee {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub user_cross_rate: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub active_referral_discount: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerUpdate {
    pub time: u64,
    pub hash: String,
    pub delta: LedgerDelta,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LedgerDelta {
    Send {
        nonce: u64,
    },
    SpotTransfer {
        nonce: u64,
    },
    CStakingTransfer {
        token: String,
        amount: String,
        is_deposit: bool,
    },
    #[serde(other)]
    Other,
}
