// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    annotated_value::MoveStructLayout,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::{balance::Balance, base_types::ObjectID, coin::Coin, id::UID, SUI_FRAMEWORK_ADDRESS};

/// The number of Mist per Sui token
pub const MIST_PER_SUI: u64 = 1_000_000_000;

/// Total supply denominated in Sui
pub const TOTAL_SUPPLY_SUI: u64 = 10_000_000_000;

// Note: cannot use checked arithmetic here since `const unwrap` is still unstable.
/// Total supply denominated in Mist
pub const TOTAL_SUPPLY_MIST: u64 = TOTAL_SUPPLY_SUI * MIST_PER_SUI;

pub const GAS_MODULE_NAME: &IdentStr = ident_str!("sui");
pub const GAS_STRUCT_NAME: &IdentStr = ident_str!("SUI");

pub use checked::*;

#[sui_macros::with_checked_arithmetic]
mod checked {
    use super::*;

    pub struct GAS {}
    impl GAS {
        pub fn type_() -> StructTag {
            StructTag {
                address: SUI_FRAMEWORK_ADDRESS,
                name: GAS_STRUCT_NAME.to_owned(),
                module: GAS_MODULE_NAME.to_owned(),
                type_params: Vec::new(),
            }
        }

        pub fn type_tag() -> TypeTag {
            TypeTag::Struct(Box::new(Self::type_()))
        }

        pub fn is_gas(other: &StructTag) -> bool {
            &Self::type_() == other
        }

        pub fn is_gas_type(other: &TypeTag) -> bool {
            match other {
                TypeTag::Struct(s) => Self::is_gas(s),
                _ => false,
            }
        }
    }

    /// Rust version of the Move sui::coin::Coin<Sui::sui::SUI> type
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct GasCoin(pub Coin);

    impl GasCoin {
        pub fn new(id: ObjectID, value: u64) -> Self {
            Self(Coin::new(UID::new(id), value))
        }

        pub fn value(&self) -> u64 {
            self.0.value()
        }

        pub fn type_() -> StructTag {
            Coin::type_(TypeTag::Struct(Box::new(GAS::type_())))
        }

        /// Return `true` if `s` is the type of a gas coin (i.e., 0x2::coin::Coin<0x2::sui::SUI>)
        pub fn is_gas_coin(s: &StructTag) -> bool {
            Coin::is_coin(s) && s.type_params.len() == 1 && GAS::is_gas_type(&s.type_params[0])
        }

        /// Return `true` if `s` is the type of a gas balance (i.e., 0x2::balance::Balance<0x2::sui::SUI>)
        pub fn is_gas_balance(s: &StructTag) -> bool {
            Balance::is_balance(s)
                && s.type_params.len() == 1
                && GAS::is_gas_type(&s.type_params[0])
        }

        pub fn id(&self) -> &ObjectID {
            self.0.id()
        }

        pub fn to_bcs_bytes(&self) -> Vec<u8> {
            bcs::to_bytes(&self).unwrap()
        }

        // pub fn to_object(&self, version: SequenceNumber) -> MoveObject {
        //     MoveObject::new_gas_coin(version, *self.id(), self.value())
        // }

        pub fn layout() -> MoveStructLayout {
            Coin::layout(TypeTag::Struct(Box::new(GAS::type_())))
        }

        #[cfg(any(feature = "test-utils", test))]
        pub fn new_for_testing(value: u64) -> Self {
            Self::new(ObjectID::random(), value)
        }

        #[cfg(any(feature = "test-utils", test))]
        pub fn new_for_testing_with_id(id: ObjectID, value: u64) -> Self {
            Self::new(id, value)
        }
    }

    impl Display for GasCoin {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Coin {{ id: {}, value: {} }}", self.id(), self.value())
        }
    }
}
