use crate::signer::AccountAddress;
use serde::ser::{SerializeSeq, Serializer};
use serde::Serialize;

#[derive(Clone, Debug)]
pub(crate) enum MoveValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256([u8; 32]),
    Address(AccountAddress),
    Signer(AccountAddress),
    Vector(Vec<MoveValue>),
}

impl Serialize for MoveValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MoveValue::Bool(value) => serializer.serialize_bool(*value),
            MoveValue::U8(value) => serializer.serialize_u8(*value),
            MoveValue::U16(value) => serializer.serialize_u16(*value),
            MoveValue::U32(value) => serializer.serialize_u32(*value),
            MoveValue::U64(value) => serializer.serialize_u64(*value),
            MoveValue::U128(value) => serializer.serialize_u128(*value),
            MoveValue::U256(value) => value.serialize(serializer),
            MoveValue::Address(value) => value.serialize(serializer),
            MoveValue::Signer(value) => value.serialize(serializer),
            MoveValue::Vector(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
        }
    }
}
