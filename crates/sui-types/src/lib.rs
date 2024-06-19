// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

pub mod balance;
pub mod base_types;
pub mod coin;
pub mod crypto;
pub mod digests;
pub mod error;
pub mod execution_status;
pub mod fastcrypto;
pub mod gas_coin;
pub mod governance;
pub mod id;
pub mod intent;
pub mod move_package;
pub mod object;
pub mod programmable_transaction_builder;
pub mod sui_serde;
pub mod transaction;
use base_types::{ObjectID, SequenceNumber, SuiAddress};
use move_core_types::account_address::AccountAddress;
pub use move_core_types::{identifier::Identifier, language_storage::TypeTag};
use object::OBJECT_START_VERSION;

macro_rules! built_in_ids {
    ($($addr:ident / $id:ident = $init:expr);* $(;)?) => {
        $(
            pub const $addr: AccountAddress = builtin_address($init);
            pub const $id: ObjectID = ObjectID::from_address($addr);
        )*
    }
}

macro_rules! built_in_pkgs {
    ($($addr:ident / $id:ident = $init:expr);* $(;)?) => {
        built_in_ids! { $($addr / $id = $init;)* }
        pub const SYSTEM_PACKAGE_ADDRESSES: &[AccountAddress] = &[$($addr),*];
        pub fn is_system_package(addr: impl Into<AccountAddress>) -> bool {
            matches!(addr.into(), $($addr)|*)
        }
    }
}

built_in_pkgs! {
    MOVE_STDLIB_ADDRESS / MOVE_STDLIB_PACKAGE_ID = 0x1;
    SUI_FRAMEWORK_ADDRESS / SUI_FRAMEWORK_PACKAGE_ID = 0x2;
    SUI_SYSTEM_ADDRESS / SUI_SYSTEM_PACKAGE_ID = 0x3;
    BRIDGE_ADDRESS / BRIDGE_PACKAGE_ID = 0xb;
    DEEPBOOK_ADDRESS / DEEPBOOK_PACKAGE_ID = 0xdee9;
}

built_in_ids! {
    SUI_SYSTEM_STATE_ADDRESS / SUI_SYSTEM_STATE_OBJECT_ID = 0x5;
    SUI_CLOCK_ADDRESS / SUI_CLOCK_OBJECT_ID = 0x6;
    SUI_AUTHENTICATOR_STATE_ADDRESS / SUI_AUTHENTICATOR_STATE_OBJECT_ID = 0x7;
    SUI_RANDOMNESS_STATE_ADDRESS / SUI_RANDOMNESS_STATE_OBJECT_ID = 0x8;
    SUI_BRIDGE_ADDRESS / SUI_BRIDGE_OBJECT_ID = 0x9;
    SUI_DENY_LIST_ADDRESS / SUI_DENY_LIST_OBJECT_ID = 0x403;
}

pub const SUI_SYSTEM_STATE_OBJECT_SHARED_VERSION: SequenceNumber = OBJECT_START_VERSION;
pub const SUI_CLOCK_OBJECT_SHARED_VERSION: SequenceNumber = OBJECT_START_VERSION;
pub const SUI_AUTHENTICATOR_STATE_OBJECT_SHARED_VERSION: SequenceNumber = OBJECT_START_VERSION;

const fn builtin_address(suffix: u16) -> AccountAddress {
    let mut addr = [0u8; AccountAddress::LENGTH];
    let [hi, lo] = suffix.to_be_bytes();
    addr[AccountAddress::LENGTH - 2] = hi;
    addr[AccountAddress::LENGTH - 1] = lo;
    AccountAddress::new(addr)
}

pub trait MoveTypeTagTrait {
    fn get_type_tag() -> TypeTag;
}

impl MoveTypeTagTrait for u8 {
    fn get_type_tag() -> TypeTag {
        TypeTag::U8
    }
}

impl MoveTypeTagTrait for u64 {
    fn get_type_tag() -> TypeTag {
        TypeTag::U64
    }
}

impl MoveTypeTagTrait for ObjectID {
    fn get_type_tag() -> TypeTag {
        TypeTag::Address
    }
}

impl MoveTypeTagTrait for SuiAddress {
    fn get_type_tag() -> TypeTag {
        TypeTag::Address
    }
}

impl<T: MoveTypeTagTrait> MoveTypeTagTrait for Vec<T> {
    fn get_type_tag() -> TypeTag {
        TypeTag::Vector(Box::new(T::get_type_tag()))
    }
}
