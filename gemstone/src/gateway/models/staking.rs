use primitives::{AssetId, Chain, DelegationBase, DelegationValidator};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commission: f64,
    pub apr: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationBase {
    pub asset_id: AssetId,
    pub delegation_id: String,
    pub validator_id: String,
    pub balance: String,
    pub shares: String,
    pub completion_date: Option<u64>,
    pub delegation_state: String,
    pub rewards: String,
}

impl From<DelegationValidator> for GemDelegationValidator {
    fn from(validator: DelegationValidator) -> Self {
        Self {
            chain: validator.chain,
            id: validator.id,
            name: validator.name,
            is_active: validator.is_active,
            commission: validator.commision,
            apr: validator.apr,
        }
    }
}

impl From<DelegationBase> for GemDelegationBase {
    fn from(delegation: DelegationBase) -> Self {
        Self {
            asset_id: delegation.asset_id,
            delegation_id: delegation.delegation_id,
            validator_id: delegation.validator_id,
            balance: delegation.balance,
            shares: delegation.shares,
            completion_date: delegation.completion_date.map(|dt| dt.timestamp() as u64),
            delegation_state: delegation.state.as_ref().to_string(),
            rewards: delegation.rewards,
        }
    }
}
