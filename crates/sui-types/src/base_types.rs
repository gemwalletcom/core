// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::coin::{Coin, CoinMetadata, TreasuryCap, COIN_MODULE_NAME, COIN_STRUCT_NAME};
use crate::error::SuiError;
use crate::fastcrypto::encoding::{Encoding, Hex};
use crate::gas_coin::{GasCoin, GAS};
use crate::governance::{StakedSui, STAKED_SUI_STRUCT_NAME, STAKING_POOL_MODULE_NAME};
use crate::sui_serde::{HexAccountAddress, Readable};
use crate::SUI_SYSTEM_ADDRESS;
use anyhow::{anyhow, Error};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{fmt, str::FromStr};

pub use crate::digests::ObjectDigest;

const SUI_FRAMEWORK_ADDRESS: AccountAddress = address_from_single_byte(2);

const fn address_from_single_byte(b: u8) -> AccountAddress {
    let mut addr = [0u8; AccountAddress::LENGTH];
    addr[AccountAddress::LENGTH - 1] = b;
    AccountAddress::new(addr)
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectID(#[serde_as(as = "Readable<HexAccountAddress, _>")] AccountAddress);

#[derive(
    Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Default, Debug, Serialize, Deserialize,
)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub const MIN: SequenceNumber = SequenceNumber(u64::MIN);
    pub const MAX: SequenceNumber = SequenceNumber(0x7fff_ffff_ffff_ffff);
    pub const CANCELLED_READ: SequenceNumber = SequenceNumber(SequenceNumber::MAX.value() + 1);
    pub const CONGESTED: SequenceNumber = SequenceNumber(SequenceNumber::MAX.value() + 2);

    pub const fn new() -> Self {
        SequenceNumber(0)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }

    pub const fn from_u64(u: u64) -> Self {
        SequenceNumber(u)
    }

    pub fn increment(&mut self) {
        assert_ne!(self.0, u64::MAX);
        self.0 += 1;
    }

    pub fn decrement(&mut self) {
        assert_ne!(self.0, 0);
        self.0 -= 1;
    }
}

impl From<SequenceNumber> for u64 {
    fn from(val: SequenceNumber) -> Self {
        val.0
    }
}

impl From<u64> for SequenceNumber {
    fn from(value: u64) -> Self {
        SequenceNumber(value)
    }
}

impl From<SequenceNumber> for usize {
    fn from(value: SequenceNumber) -> Self {
        value.0 as usize
    }
}

pub type ObjectRef = (ObjectID, SequenceNumber, ObjectDigest);

pub const SUI_ADDRESS_LENGTH: usize = ObjectID::LENGTH;

#[serde_as]
#[derive(Eq, Default, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct SuiAddress(#[serde_as(as = "Readable<Hex, _>")] [u8; SUI_ADDRESS_LENGTH]);

impl SuiAddress {
    pub const ZERO: Self = Self([0u8; SUI_ADDRESS_LENGTH]);

    /// Convert the address to a byte buffer.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    #[cfg(any(feature = "test-utils", test))]
    /// Return a random SuiAddress.
    pub fn random_for_testing_only() -> Self {
        AccountAddress::random().into()
    }

    // pub fn generate<R: rand::RngCore + rand::CryptoRng>(mut rng: R) -> Self {
    //     let buf: [u8; SUI_ADDRESS_LENGTH] = rng.gen();
    //     Self(buf)
    // }

    /// Serialize an `Option<SuiAddress>` in Hex.
    pub fn optional_address_as_hex<S>(
        key: &Option<SuiAddress>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&key.map(Hex::encode).unwrap_or_default())
    }

    /// Deserialize into an `Option<SuiAddress>`.
    pub fn optional_address_from_hex<'de, D>(
        deserializer: D,
    ) -> Result<Option<SuiAddress>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value = decode_bytes_hex(&s).map_err(serde::de::Error::custom)?;
        Ok(Some(value))
    }

    /// Return the underlying byte array of a SuiAddress.
    pub fn to_inner(self) -> [u8; SUI_ADDRESS_LENGTH] {
        self.0
    }

    /// Parse a SuiAddress from a byte array or buffer.
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, SuiError> {
        <[u8; SUI_ADDRESS_LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| SuiError::InvalidAddress)
            .map(SuiAddress)
    }
}

impl From<ObjectID> for SuiAddress {
    fn from(object_id: ObjectID) -> SuiAddress {
        Self(object_id.into_bytes())
    }
}

impl From<AccountAddress> for SuiAddress {
    fn from(address: AccountAddress) -> SuiAddress {
        Self(address.into_bytes())
    }
}

impl TryFrom<&[u8]> for SuiAddress {
    type Error = SuiError;

    /// Tries to convert the provided byte array into a SuiAddress.
    fn try_from(bytes: &[u8]) -> Result<Self, SuiError> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for SuiAddress {
    type Error = SuiError;

    /// Tries to convert the provided byte buffer into a SuiAddress.
    fn try_from(bytes: Vec<u8>) -> Result<Self, SuiError> {
        Self::from_bytes(bytes)
    }
}

impl AsRef<[u8]> for SuiAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl FromStr for SuiAddress {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        decode_bytes_hex(s).map_err(|e| anyhow!(e))
    }
}

impl fmt::Display for SuiAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", Hex::encode(self.0))
    }
}

impl fmt::Debug for SuiAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "0x{}", Hex::encode(self.0))
    }
}

impl ObjectID {
    /// The number of bytes in an address.
    pub const LENGTH: usize = AccountAddress::LENGTH;
    /// Hex address: 0x0
    pub const ZERO: Self = Self::new([0u8; Self::LENGTH]);
    pub const MAX: Self = Self::new([0xff; Self::LENGTH]);
    /// Create a new ObjectID
    pub const fn new(obj_id: [u8; Self::LENGTH]) -> Self {
        Self(AccountAddress::new(obj_id))
    }

    /// Const fn variant of `<ObjectID as From<AccountAddress>>::from`
    pub const fn from_address(addr: AccountAddress) -> Self {
        Self(addr)
    }

    /// Return a random ObjectID.
    pub fn random() -> Self {
        Self::from(AccountAddress::random())
    }

    /// Return the underlying bytes buffer of the ObjectID.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Parse the ObjectID from byte array or buffer.
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, ObjectIDParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| ObjectIDParseError::TryFromSliceError)
            .map(ObjectID::new)
    }

    /// Return the underlying bytes array of the ObjectID.
    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0.into_bytes()
    }

    /// Make an ObjectID with padding 0s before the single byte.
    pub const fn from_single_byte(byte: u8) -> ObjectID {
        let mut bytes = [0u8; Self::LENGTH];
        bytes[Self::LENGTH - 1] = byte;
        ObjectID::new(bytes)
    }

    /// Convert from hex string to ObjectID where the string is prefixed with 0x
    /// Padding 0s if the string is too short.
    pub fn from_hex_literal(literal: &str) -> Result<Self, ObjectIDParseError> {
        if !literal.starts_with("0x") {
            return Err(ObjectIDParseError::HexLiteralPrefixMissing);
        }

        let hex_len = literal.len() - 2;

        // If the string is too short, pad it
        if hex_len < Self::LENGTH * 2 {
            let mut hex_str = String::with_capacity(Self::LENGTH * 2);
            for _ in 0..Self::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(&literal[2..]);
            Self::from_str(&hex_str)
        } else {
            Self::from_str(&literal[2..])
        }
    }

    /// Incremenent the ObjectID by usize IDs, assuming the ObjectID hex is a number represented as an array of bytes
    pub fn advance(&self, step: usize) -> Result<ObjectID, anyhow::Error> {
        let mut curr_vec = self.to_vec();
        let mut step_copy = step;

        let mut carry = 0;
        for idx in (0..Self::LENGTH).rev() {
            if step_copy == 0 {
                // Nothing else to do
                break;
            }
            // Extract the relevant part
            let g = (step_copy % 0x100) as u16;
            // Shift to next group
            step_copy >>= 8;
            let mut val = curr_vec[idx] as u16;
            (carry, val) = ((val + carry + g) / 0x100, (val + carry + g) % 0x100);
            curr_vec[idx] = val as u8;
        }

        if carry > 0 {
            return Err(anyhow!("Increment will cause overflow"));
        }
        ObjectID::try_from(curr_vec).map_err(|w| w.into())
    }

    /// Increment the ObjectID by one, assuming the ObjectID hex is a number represented as an array of bytes
    pub fn next_increment(&self) -> Result<ObjectID, anyhow::Error> {
        let mut prev_val = self.to_vec();
        let mx = [0xFF; Self::LENGTH];

        if prev_val == mx {
            return Err(anyhow!("Increment will cause overflow"));
        }

        // This logic increments the integer representation of an ObjectID u8 array
        for idx in (0..Self::LENGTH).rev() {
            if prev_val[idx] == 0xFF {
                prev_val[idx] = 0;
            } else {
                prev_val[idx] += 1;
                break;
            };
        }
        ObjectID::try_from(prev_val.clone()).map_err(|w| w.into())
    }

    /// Create `count` object IDs starting with one at `offset`
    pub fn in_range(offset: ObjectID, count: u64) -> Result<Vec<ObjectID>, anyhow::Error> {
        let mut ret = Vec::new();
        let mut prev = offset;
        for o in 0..count {
            if o != 0 {
                prev = prev.next_increment()?;
            }
            ret.push(prev);
        }
        Ok(ret)
    }

    /// Return the full hex string with 0x prefix without removing trailing 0s. Prefer this
    /// over [fn to_hex_literal] if the string needs to be fully preserved.
    pub fn to_hex_uncompressed(&self) -> String {
        format!("{self}")
    }
}

impl From<SuiAddress> for ObjectID {
    fn from(address: SuiAddress) -> ObjectID {
        let tmp: AccountAddress = address.into();
        tmp.into()
    }
}

impl From<AccountAddress> for ObjectID {
    fn from(address: AccountAddress) -> Self {
        Self(address)
    }
}

impl fmt::Display for ObjectID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "0x{}", Hex::encode(self.0))
    }
}

impl fmt::Debug for ObjectID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "0x{}", Hex::encode(self.0))
    }
}

impl AsRef<[u8]> for ObjectID {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl TryFrom<&[u8]> for ObjectID {
    type Error = ObjectIDParseError;

    /// Tries to convert the provided byte array into ObjectID.
    fn try_from(bytes: &[u8]) -> Result<ObjectID, ObjectIDParseError> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for ObjectID {
    type Error = ObjectIDParseError;

    /// Tries to convert the provided byte buffer into ObjectID.
    fn try_from(bytes: Vec<u8>) -> Result<ObjectID, ObjectIDParseError> {
        Self::from_bytes(bytes)
    }
}

impl FromStr for ObjectID {
    type Err = ObjectIDParseError;

    /// Parse ObjectID from hex string with or without 0x prefix, pad with 0s if needed.
    fn from_str(s: &str) -> Result<Self, ObjectIDParseError> {
        decode_bytes_hex(s).or_else(|_| Self::from_hex_literal(s))
    }
}

impl std::ops::Deref for ObjectID {
    type Target = AccountAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Decodes a hex string to bytes. Both upper and lower case characters are allowed in the hex string.
pub fn decode_bytes_hex<T: for<'a> TryFrom<&'a [u8]>>(s: &str) -> Result<T, Error> {
    let value = Hex::decode(s)?;
    T::try_from(&value[..]).map_err(|_| anyhow!("InvalidInput"))
}

#[derive(PartialEq, Eq, Clone, Debug, thiserror::Error)]
pub enum ObjectIDParseError {
    #[error("ObjectID hex literal must start with 0x")]
    HexLiteralPrefixMissing,

    #[error("Could not convert from bytes slice")]
    TryFromSliceError,
}

impl From<ObjectID> for AccountAddress {
    fn from(obj_id: ObjectID) -> Self {
        obj_id.0
    }
}

impl From<SuiAddress> for AccountAddress {
    fn from(address: SuiAddress) -> Self {
        Self::new(address.0)
    }
}

/// Wrapper around StructTag with a space-efficient representation for common types like coins
/// The StructTag for a gas coin is 84 bytes, so using 1 byte instead is a win.
/// The inner representation is private to prevent incorrectly constructing an `Other` instead of
/// one of the specialized variants, e.g. `Other(GasCoin::type_())` instead of `GasCoin`
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveObjectType(MoveObjectType_);

/// Even though it is declared public, it is the "private", internal representation for
/// `MoveObjectType`
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Deserialize, Serialize, Hash)]
pub enum MoveObjectType_ {
    /// A type that is not `0x2::coin::Coin<T>`
    Other(StructTag),
    /// A SUI coin (i.e., `0x2::coin::Coin<0x2::sui::SUI>`)
    GasCoin,
    /// A record of a staked SUI coin (i.e., `0x3::staking_pool::StakedSui`)
    StakedSui,
    /// A non-SUI coin type (i.e., `0x2::coin::Coin<T> where T != 0x2::sui::SUI`)
    Coin(TypeTag),
    // NOTE: if adding a new type here, and there are existing on-chain objects of that
    // type with Other(_), that is ok, but you must hand-roll PartialEq/Eq/Ord/maybe Hash
    // to make sure the new type and Other(_) are interpreted consistently.
}

impl MoveObjectType {
    pub fn gas_coin() -> Self {
        Self(MoveObjectType_::GasCoin)
    }

    pub fn staked_sui() -> Self {
        Self(MoveObjectType_::StakedSui)
    }

    pub fn address(&self) -> AccountAddress {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => SUI_FRAMEWORK_ADDRESS,
            MoveObjectType_::StakedSui => SUI_SYSTEM_ADDRESS,
            MoveObjectType_::Other(s) => s.address,
        }
    }

    pub fn module(&self) -> &IdentStr {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => COIN_MODULE_NAME,
            MoveObjectType_::StakedSui => STAKING_POOL_MODULE_NAME,
            MoveObjectType_::Other(s) => &s.module,
        }
    }

    pub fn name(&self) -> &IdentStr {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => COIN_STRUCT_NAME,
            MoveObjectType_::StakedSui => STAKED_SUI_STRUCT_NAME,
            MoveObjectType_::Other(s) => &s.name,
        }
    }

    pub fn type_params(&self) -> Vec<TypeTag> {
        match &self.0 {
            MoveObjectType_::GasCoin => vec![GAS::type_tag()],
            MoveObjectType_::StakedSui => vec![],
            MoveObjectType_::Coin(inner) => vec![inner.clone()],
            MoveObjectType_::Other(s) => s.type_params.clone(),
        }
    }

    pub fn into_type_params(self) -> Vec<TypeTag> {
        match self.0 {
            MoveObjectType_::GasCoin => vec![GAS::type_tag()],
            MoveObjectType_::StakedSui => vec![],
            MoveObjectType_::Coin(inner) => vec![inner],
            MoveObjectType_::Other(s) => s.type_params,
        }
    }

    pub fn coin_type_maybe(&self) -> Option<TypeTag> {
        match &self.0 {
            MoveObjectType_::GasCoin => Some(GAS::type_tag()),
            MoveObjectType_::Coin(inner) => Some(inner.clone()),
            MoveObjectType_::StakedSui => None,
            MoveObjectType_::Other(_) => None,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        ModuleId::new(self.address(), self.module().to_owned())
    }

    pub fn size_for_gas_metering(&self) -> usize {
        // unwraps safe because a `StructTag` cannot fail to serialize
        match &self.0 {
            MoveObjectType_::GasCoin => 1,
            MoveObjectType_::StakedSui => 1,
            MoveObjectType_::Coin(inner) => bcs::serialized_size(inner).unwrap() + 1,
            MoveObjectType_::Other(s) => bcs::serialized_size(s).unwrap() + 1,
        }
    }

    /// Return true if `self` is `0x2::coin::Coin<T>` for some T (note: T can be SUI)
    pub fn is_coin(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) => true,
            MoveObjectType_::StakedSui | MoveObjectType_::Other(_) => false,
        }
    }

    /// Return true if `self` is 0x2::coin::Coin<0x2::sui::SUI>
    pub fn is_gas_coin(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin => true,
            MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) | MoveObjectType_::Other(_) => {
                false
            }
        }
    }

    /// Return true if `self` is `0x2::coin::Coin<t>`
    pub fn is_coin_t(&self, t: &TypeTag) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin => GAS::is_gas_type(t),
            MoveObjectType_::Coin(c) => t == c,
            MoveObjectType_::StakedSui | MoveObjectType_::Other(_) => false,
        }
    }

    pub fn is_staked_sui(&self) -> bool {
        match &self.0 {
            MoveObjectType_::StakedSui => true,
            MoveObjectType_::GasCoin | MoveObjectType_::Coin(_) | MoveObjectType_::Other(_) => {
                false
            }
        }
    }

    pub fn is_coin_metadata(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
                false
            }
            MoveObjectType_::Other(s) => CoinMetadata::is_coin_metadata(s),
        }
    }

    pub fn is_treasury_cap(&self) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
                false
            }
            MoveObjectType_::Other(s) => TreasuryCap::is_treasury_type(s),
        }
    }

    pub fn is_upgrade_cap(&self) -> bool {
        self.address() == SUI_FRAMEWORK_ADDRESS
            && self.module().as_str() == "package"
            && self.name().as_str() == "UpgradeCap"
    }

    pub fn is_regulated_coin_metadata(&self) -> bool {
        self.address() == SUI_FRAMEWORK_ADDRESS
            && self.module().as_str() == "coin"
            && self.name().as_str() == "RegulatedCoinMetadata"
    }

    pub fn is_coin_deny_cap(&self) -> bool {
        self.address() == SUI_FRAMEWORK_ADDRESS
            && self.module().as_str() == "coin"
            && self.name().as_str() == "DenyCap"
    }

    // pub fn is_dynamic_field(&self) -> bool {
    //     match &self.0 {
    //         MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
    //             false
    //         }
    //         MoveObjectType_::Other(s) => DynamicFieldInfo::is_dynamic_field(s),
    //     }
    // }

    // pub fn try_extract_field_name(&self, type_: &DynamicFieldType) -> SuiResult<TypeTag> {
    //     match &self.0 {
    //         MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
    //             Err(SuiError::ObjectDeserializationError {
    //                 error: "Error extracting dynamic object name from Coin object".to_string(),
    //             })
    //         }
    //         MoveObjectType_::Other(s) => DynamicFieldInfo::try_extract_field_name(s, type_),
    //     }
    // }

    // pub fn try_extract_field_value(&self) -> SuiResult<TypeTag> {
    //     match &self.0 {
    //         MoveObjectType_::GasCoin | MoveObjectType_::StakedSui | MoveObjectType_::Coin(_) => {
    //             Err(SuiError::ObjectDeserializationError {
    //                 error: "Error extracting dynamic object value from Coin object".to_string(),
    //             })
    //         }
    //         MoveObjectType_::Other(s) => DynamicFieldInfo::try_extract_field_value(s),
    //     }
    // }

    pub fn is(&self, s: &StructTag) -> bool {
        match &self.0 {
            MoveObjectType_::GasCoin => GasCoin::is_gas_coin(s),
            MoveObjectType_::StakedSui => StakedSui::is_staked_sui(s),
            MoveObjectType_::Coin(inner) => {
                Coin::is_coin(s) && s.type_params.len() == 1 && inner == &s.type_params[0]
            }
            MoveObjectType_::Other(o) => s == o,
        }
    }

    /// Returns the string representation of this object's type using the canonical display.
    pub fn to_canonical_string(&self, with_prefix: bool) -> String {
        StructTag::from(self.clone()).to_canonical_string(with_prefix)
    }
}

impl From<StructTag> for MoveObjectType {
    fn from(mut s: StructTag) -> Self {
        Self(if GasCoin::is_gas_coin(&s) {
            MoveObjectType_::GasCoin
        } else if Coin::is_coin(&s) {
            // unwrap safe because a coin has exactly one type parameter
            MoveObjectType_::Coin(s.type_params.pop().unwrap())
        } else if StakedSui::is_staked_sui(&s) {
            MoveObjectType_::StakedSui
        } else {
            MoveObjectType_::Other(s)
        })
    }
}

impl From<MoveObjectType> for StructTag {
    fn from(t: MoveObjectType) -> Self {
        match t.0 {
            MoveObjectType_::GasCoin => GasCoin::type_(),
            MoveObjectType_::StakedSui => StakedSui::type_(),
            MoveObjectType_::Coin(inner) => Coin::type_(inner),
            MoveObjectType_::Other(s) => s,
        }
    }
}

impl From<MoveObjectType> for TypeTag {
    fn from(o: MoveObjectType) -> TypeTag {
        let s: StructTag = o.into();
        TypeTag::Struct(Box::new(s))
    }
}
