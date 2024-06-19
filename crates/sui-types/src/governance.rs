// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::balance::Balance;
use crate::base_types::ObjectID;
use crate::id::{ID, UID};
use crate::SUI_SYSTEM_ADDRESS;
use move_core_types::{ident_str, identifier::IdentStr, language_storage::StructTag};
use serde::{Deserialize, Serialize};

pub const STAKING_POOL_MODULE_NAME: &IdentStr = ident_str!("staking_pool");
pub const STAKED_SUI_STRUCT_NAME: &IdentStr = ident_str!("StakedSui");
pub const ADD_STAKE_MUL_COIN_FUN_NAME: &IdentStr = ident_str!("request_add_stake_mul_coin");
pub const ADD_STAKE_FUN_NAME: &IdentStr = ident_str!("request_add_stake");
pub const WITHDRAW_STAKE_FUN_NAME: &IdentStr = ident_str!("request_withdraw_stake");

pub type EpochId = u64;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct StakedSui {
    id: UID,
    pool_id: ID,
    stake_activation_epoch: u64,
    principal: Balance,
}

impl StakedSui {
    pub fn type_() -> StructTag {
        StructTag {
            address: SUI_SYSTEM_ADDRESS,
            module: STAKING_POOL_MODULE_NAME.to_owned(),
            name: STAKED_SUI_STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }

    pub fn is_staked_sui(s: &StructTag) -> bool {
        s.address == SUI_SYSTEM_ADDRESS
            && s.module.as_ident_str() == STAKING_POOL_MODULE_NAME
            && s.name.as_ident_str() == STAKED_SUI_STRUCT_NAME
            && s.type_params.is_empty()
    }

    pub fn id(&self) -> ObjectID {
        self.id.id.bytes
    }

    pub fn pool_id(&self) -> ObjectID {
        self.pool_id.bytes
    }

    pub fn activation_epoch(&self) -> EpochId {
        self.stake_activation_epoch
    }

    pub fn request_epoch(&self) -> EpochId {
        // TODO: this might change when we implement warm up period.
        self.stake_activation_epoch.saturating_sub(1)
    }

    pub fn principal(&self) -> u64 {
        self.principal.value()
    }
}
