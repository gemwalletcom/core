// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use blake2::{digest::consts::U32, Blake2b};

pub type Blake2b256 = Blake2b<U32>;
pub type DefaultHash = Blake2b256;
