// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error)]
pub enum ExecutionFailureStatus {
    //
    // General transaction errors
    //
    #[error("Insufficient Gas.")]
    InsufficientGas,

    //
    // Coin errors
    //
    #[error("Insufficient coin balance for operation.")]
    InsufficientCoinBalance,
    #[error("The coin balance overflows u64")]
    CoinBalanceOverflow,
}

pub type CommandIndex = usize;
