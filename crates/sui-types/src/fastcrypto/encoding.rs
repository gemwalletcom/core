// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::fastcrypto::error::{FastCryptoError, FastCryptoError::InvalidInput, FastCryptoResult};
use serde;
use serde::de::{Deserializer, Error};
use serde::ser::Serializer;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{DeserializeAs, SerializeAs};

/// Trait representing a general binary-to-string encoding.
pub trait Encoding {
    /// Decode this encoding into bytes.
    fn decode(s: &str) -> FastCryptoResult<Vec<u8>>;

    /// Encode bytes into a string.
    fn encode<T: AsRef<[u8]>>(data: T) -> String;
}

/// Implement `DeserializeAs<Vec<u8>>`, `DeserializeAs<[u8; N]>` and `SerializeAs<T: AsRef<[u8]>`
/// for a type that implements `Encoding`.
macro_rules! impl_serde_as_for_encoding {
    ($encoding:ty) => {
        impl<'de> DeserializeAs<'de, Vec<u8>> for $encoding {
            fn deserialize_as<D>(deserializer: D) -> Result<Vec<u8>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                Self::decode(&s).map_err(|_| Error::custom("Deserialization failed"))
            }
        }

        impl<T> SerializeAs<T> for $encoding
        where
            T: AsRef<[u8]>,
        {
            fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let encoded_string = Self::encode(value);
                Self(encoded_string).serialize(serializer)
            }
        }

        impl<'de, const N: usize> DeserializeAs<'de, [u8; N]> for $encoding {
            fn deserialize_as<D>(deserializer: D) -> Result<[u8; N], D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: Vec<u8> = <$encoding>::deserialize_as(deserializer)?;
                value
                    .try_into()
                    .map_err(|_| Error::custom(format!("Invalid array length, expecting {}", N)))
            }
        }
    };
}

/// Implement `TryFrom<String>` for a type that implements `Encoding`.
macro_rules! impl_try_from_string {
    ($encoding:ty) => {
        impl TryFrom<String> for $encoding {
            type Error = FastCryptoError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                // Error on invalid encoding
                <$encoding>::decode(&value)?;
                Ok(Self(value))
            }
        }
    };
}

/// Hex string encoding.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct Hex(String);

impl TryFrom<String> for Hex {
    type Error = FastCryptoError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.strip_prefix("0x").unwrap_or(&value);
        Ok(Self(s.to_string()))
    }
}

impl Serialize for Hex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Hex strings are serialized with a 0x prefix which differs from the output of `Hex::encode`.
        String::serialize(&self.encoded_with_format(), serializer)
    }
}

impl_serde_as_for_encoding!(Hex);

impl Hex {
    /// Create a hex encoding from a string.
    #[cfg(test)]
    pub fn from_string(s: &str) -> Self {
        Hex(s.to_string())
    }
    /// Decodes this hex encoding to bytes.
    pub fn to_vec(&self) -> FastCryptoResult<Vec<u8>> {
        Self::decode(&self.0)
    }
    /// Encodes bytes as a hex string.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Self::encode(bytes))
    }
    /// Encode bytes as a hex string with a "0x" prefix.
    pub fn encode_with_format<T: AsRef<[u8]>>(bytes: T) -> String {
        Self::format(&Self::encode(bytes))
    }
    /// Get a string representation of this Hex encoding with a "0x" prefix.
    pub fn encoded_with_format(&self) -> String {
        Self::format(&self.0)
    }
    /// Add "0x" prefix to a hex string.
    fn format(hex_string: &str) -> String {
        format!("0x{}", hex_string)
    }
}

/// Decodes a hex string to bytes. Both upper and lower case characters are allowed in the hex string.
pub fn decode_bytes_hex<T: for<'a> TryFrom<&'a [u8]>>(s: &str) -> FastCryptoResult<T> {
    let value = Hex::decode(s)?;
    T::try_from(&value[..]).map_err(|_| InvalidInput)
}

impl Encoding for Hex {
    /// Decodes a hex string to bytes. Both upper and lower case characters are accepted, and the
    /// string may have a "0x" prefix or not.
    fn decode(s: &str) -> FastCryptoResult<Vec<u8>> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        hex::decode(s).map_err(|_| InvalidInput)
    }

    /// Hex encoding is without "0x" prefix. See `Hex::encode_with_format` for encoding with "0x".
    fn encode<T: AsRef<[u8]>>(data: T) -> String {
        hex::encode(data.as_ref())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(try_from = "String")]
pub struct Base58(String);

impl_serde_as_for_encoding!(Base58);
impl_try_from_string!(Base58);

impl Encoding for Base58 {
    fn decode(s: &str) -> FastCryptoResult<Vec<u8>> {
        bs58::decode(s).into_vec().map_err(|_| InvalidInput)
    }

    fn encode<T: AsRef<[u8]>>(data: T) -> String {
        bs58::encode(data).into_string()
    }
}
