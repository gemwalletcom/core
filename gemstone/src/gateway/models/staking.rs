use primitives::{DelegationBase, DelegationValidator};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationValidator {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commission: f64,
    pub apr: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationBase {
    pub delegation_id: String,
    pub validator_id: String,
    pub balance: String,
    pub completion_date: Option<u64>,
    pub delegation_state: String,
    pub rewards: String,
}

impl From<DelegationValidator> for GemDelegationValidator {
    fn from(validator: DelegationValidator) -> Self {
        Self {
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
            delegation_id: delegation.delegation_id,
            validator_id: delegation.validator_id,
            balance: delegation.balance,
            completion_date: delegation.completion_date.map(|dt| dt.timestamp() as u64),
            delegation_state: delegation.state.as_ref().to_string(),
            rewards: delegation.rewards,
        }
    }
}
