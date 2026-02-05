use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use crate::earn_position_base::{EarnPositionBase, EarnPositionState};
use crate::earn_provider::{EarnProvider, EarnProviderType};
use crate::{AssetId, Chain, Price, StakeValidator};

#[deprecated(since = "1.0.0", note = "Use EarnPosition instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub base: DelegationBase,
    pub validator: DelegationValidator,
    pub price: Option<Price>,
}

#[deprecated(since = "1.0.0", note = "Use EarnPositionBase instead")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct DelegationBase {
    pub asset_id: AssetId,
    pub state: DelegationState,
    pub balance: BigUint,
    pub shares: BigUint,
    pub rewards: BigUint,
    pub completion_date: Option<DateTime<Utc>>,
    pub delegation_id: String,
    pub validator_id: String,
}

impl From<DelegationValidator> for StakeValidator {
    fn from(value: DelegationValidator) -> Self {
        StakeValidator::new(value.id, value.name)
    }
}

#[deprecated(since = "1.0.0", note = "Use EarnProvider instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct DelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commission: f64,
    pub apr: f64,
}

#[deprecated(since = "1.0.0", note = "Use EarnPositionState instead")]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DelegationState {
    Active,
    Pending,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}

impl From<DelegationState> for EarnPositionState {
    fn from(state: DelegationState) -> Self {
        match state {
            DelegationState::Active => EarnPositionState::Active,
            DelegationState::Pending => EarnPositionState::Pending,
            DelegationState::Inactive => EarnPositionState::Inactive,
            DelegationState::Activating => EarnPositionState::Activating,
            DelegationState::Deactivating => EarnPositionState::Deactivating,
            DelegationState::AwaitingWithdrawal => EarnPositionState::AwaitingWithdrawal,
        }
    }
}

impl From<DelegationBase> for EarnPositionBase {
    fn from(base: DelegationBase) -> Self {
        Self {
            asset_id: base.asset_id,
            state: base.state.into(),
            balance: base.balance,
            shares: base.shares,
            rewards: base.rewards,
            unlock_date: base.completion_date,
            position_id: base.delegation_id,
            provider_id: base.validator_id,
        }
    }
}

impl From<DelegationValidator> for EarnProvider {
    fn from(validator: DelegationValidator) -> Self {
        Self {
            chain: validator.chain,
            id: validator.id,
            name: validator.name,
            is_active: validator.is_active,
            fee: validator.commission,
            apy: validator.apr,
            provider_type: EarnProviderType::Stake,
        }
    }
}
