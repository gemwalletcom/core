// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use blake2::{digest::consts::U32, Blake2b};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Blake2b256 = Blake2b<U32>;
pub type DefaultHash = Blake2b256;

/// A 1-byte domain separator for hashing Object ID in Sui. It is starting from 0xf0
/// to ensure no hashing collision for any ObjectID vs SuiAddress which is derived
/// as the hash of `flag || pubkey`. See `sui_types::crypto::SignatureScheme::flag()`.
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum HashingIntentScope {
    ChildObjectId = 0xf0,
    RegularObjectId = 0xf1,
}
