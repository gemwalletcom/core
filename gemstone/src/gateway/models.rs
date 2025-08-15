use primitives::{AssetBalance, AssetId, Balance, Chain};

// UniFFI wrapper types that derive from primitives
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAssetBalance {
    pub asset_id: AssetId,
    pub balance: GemBalance,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBalance {
    pub available: String,
    pub frozen: String,
    pub locked: String,
    pub staked: String,
    pub pending: String,
    pub rewards: String,
    pub reserved: String,
    pub withdrawable: String,
}

// Conversion from primitives to UniFFI types
impl From<AssetBalance> for GemAssetBalance {
    fn from(value: AssetBalance) -> Self {
        Self {
            asset_id: value.asset_id,
            balance: value.balance.into(),
        }
    }
}

impl From<Balance> for GemBalance {
    fn from(value: Balance) -> Self {
        Self {
            available: value.available,
            frozen: value.frozen,
            locked: value.locked,
            staked: value.staked,
            pending: value.pending,
            rewards: value.rewards,
            reserved: value.reserved,
            withdrawable: value.withdrawable,
        }
    }
}

impl GemBalance {
    pub fn coin_balance(available: String) -> Self {
        Self {
            available,
            frozen: "0".to_string(),
            locked: "0".to_string(),
            staked: "0".to_string(),
            pending: "0".to_string(),
            rewards: "0".to_string(),
            reserved: "0".to_string(),
            withdrawable: "0".to_string(),
        }
    }

    pub fn token_balance(available: String) -> Self {
        Self::coin_balance(available)
    }

    pub fn stake_balance(staked: String, pending: String, rewards: Option<String>) -> Self {
        Self {
            available: "0".to_string(),
            frozen: "0".to_string(),
            locked: "0".to_string(),
            staked,
            pending,
            rewards: rewards.unwrap_or("0".to_string()),
            reserved: "0".to_string(),
            withdrawable: "0".to_string(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commision: f64,
    pub apr: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationBase {
    pub asset_id: AssetId,
    pub state: String,
    pub balance: String,
    pub shares: String,
    pub rewards: String,
    pub completion_date: Option<i64>,
    pub delegation_id: String,
    pub validator_id: String,
}

impl From<primitives::DelegationValidator> for GemDelegationValidator {
    fn from(value: primitives::DelegationValidator) -> Self {
        Self {
            chain: value.chain,
            id: value.id,
            name: value.name,
            is_active: value.is_active,
            commision: value.commision,
            apr: value.apr,
        }
    }
}

impl From<primitives::DelegationBase> for GemDelegationBase {
    fn from(value: primitives::DelegationBase) -> Self {
        Self {
            asset_id: value.asset_id,
            state: value.state.as_ref().to_string(),
            balance: value.balance,
            shares: value.shares,
            rewards: value.rewards,
            completion_date: None,
            delegation_id: value.delegation_id,
            validator_id: value.validator_id,
        }
    }
}
