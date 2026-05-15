use super::{
    Field, MessageDecode, MessageEncode, MessageResult, encode_bytes_field, encode_message_field, encode_optional_bool_field, encode_optional_bytes_field,
    encode_optional_message_field, encode_optional_string_field, encode_optional_u64_field, encode_string_field,
};

pub mod optional_string {
    use super::*;

    pub fn decode(value: &mut Option<String>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.string()?);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<String>) -> Vec<u8> {
        encode_optional_string_field(field_number, value.as_deref())
    }
}

pub mod string {
    use super::*;

    pub fn decode(value: &mut String, field: Field<'_>) -> MessageResult<()> {
        *value = field.string()?;
        Ok(())
    }

    pub fn encode(field_number: u32, value: &str) -> Vec<u8> {
        encode_string_field(field_number, value)
    }
}

pub mod optional_bytes {
    use super::*;

    pub fn decode(value: &mut Option<Vec<u8>>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.bytes()?.to_vec());
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<Vec<u8>>) -> Vec<u8> {
        encode_optional_bytes_field(field_number, value.as_deref())
    }
}

pub mod optional_bool {
    use super::*;

    pub fn decode(value: &mut Option<bool>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.varint()? != 0);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<bool>) -> Vec<u8> {
        encode_optional_bool_field(field_number, *value)
    }
}

pub mod optional_varint_u32 {
    use super::*;

    pub fn decode(value: &mut Option<u32>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.varint()? as u32);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<u32>) -> Vec<u8> {
        encode_optional_u64_field(field_number, value.map(u64::from))
    }
}

pub mod optional_varint_u64 {
    use super::*;

    pub fn decode(value: &mut Option<u64>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.varint()?);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<u64>) -> Vec<u8> {
        encode_optional_u64_field(field_number, *value)
    }
}

pub mod optional_varint_i32 {
    use super::*;

    pub fn decode(value: &mut Option<i32>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.varint()? as i32);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<i32>) -> Vec<u8> {
        encode_optional_u64_field(field_number, value.map(|value| value as u64))
    }
}

pub mod optional_varint_i64 {
    use super::*;

    pub fn decode(value: &mut Option<i64>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.varint()? as i64);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &Option<i64>) -> Vec<u8> {
        encode_optional_u64_field(field_number, value.map(|value| value as u64))
    }
}

pub mod varint_i32 {
    use super::*;

    pub fn decode(value: &mut i32, field: Field<'_>) -> MessageResult<()> {
        *value = field.varint()? as i32;
        Ok(())
    }

    pub fn encode(field_number: u32, value: &i32) -> Vec<u8> {
        encode_optional_u64_field(field_number, Some(*value as u64))
    }
}

pub mod varint_i64 {
    use super::*;

    pub fn decode(value: &mut i64, field: Field<'_>) -> MessageResult<()> {
        *value = field.varint()? as i64;
        Ok(())
    }

    pub fn encode(field_number: u32, value: &i64) -> Vec<u8> {
        encode_optional_u64_field(field_number, Some(*value as u64))
    }
}

pub mod optional_enum_varint {
    use super::*;

    pub fn encode<T: Copy + Into<u64>>(field_number: u32, value: &Option<T>) -> Vec<u8> {
        encode_optional_u64_field(field_number, value.map(Into::into))
    }
}

pub mod optional_message {
    use super::*;

    pub fn decode<T: MessageDecode>(value: &mut Option<T>, field: Field<'_>) -> MessageResult<()> {
        *value = Some(field.message()?);
        Ok(())
    }

    pub fn encode<T: MessageEncode>(field_number: u32, value: &Option<T>) -> Vec<u8> {
        encode_optional_message_field(field_number, value.as_ref())
    }
}

pub mod repeated_message {
    use super::*;

    pub fn decode<T: MessageDecode>(value: &mut Vec<T>, field: Field<'_>) -> MessageResult<()> {
        value.push(field.message()?);
        Ok(())
    }

    pub fn encode<T: MessageEncode>(field_number: u32, value: &[T]) -> Vec<u8> {
        value.iter().flat_map(|value| encode_message_field(field_number, &value.encode())).collect()
    }
}

pub mod repeated_string {
    use super::*;

    pub fn decode(value: &mut Vec<String>, field: Field<'_>) -> MessageResult<()> {
        value.push(field.string()?);
        Ok(())
    }

    pub fn encode(field_number: u32, value: &[String]) -> Vec<u8> {
        value.iter().flat_map(|value| encode_string_field(field_number, value)).collect()
    }
}

pub mod repeated_bytes {
    use super::*;

    pub fn encode(field_number: u32, value: &[Vec<u8>]) -> Vec<u8> {
        value.iter().flat_map(|value| encode_bytes_field(field_number, value)).collect()
    }
}
