use super::message::{MessageDecode, MessageResult};
use super::wire::{WIRE_FIXED32, WIRE_FIXED64, WIRE_LENGTH_DELIMITED, WIRE_VARINT};

#[derive(Clone, Copy)]
pub struct Field<'a> {
    pub number: u32,
    value: FieldValue<'a>,
}

impl<'a> Field<'a> {
    pub fn varint(self) -> MessageResult<u64> {
        match self.value {
            FieldValue::Varint(value) => Ok(value),
            _ => Err(format!("protobuf field {} is not a varint", self.number).into()),
        }
    }

    pub fn fixed64(self) -> MessageResult<u64> {
        match self.value {
            FieldValue::Fixed64(value) => Ok(value),
            _ => Err(format!("protobuf field {} is not fixed64", self.number).into()),
        }
    }

    pub fn fixed32(self) -> MessageResult<u32> {
        match self.value {
            FieldValue::Fixed32(value) => Ok(value),
            _ => Err(format!("protobuf field {} is not fixed32", self.number).into()),
        }
    }

    pub fn bytes(self) -> MessageResult<&'a [u8]> {
        match self.value {
            FieldValue::Bytes(value) => Ok(value),
            _ => Err(format!("protobuf field {} is not length-delimited", self.number).into()),
        }
    }

    pub fn string(self) -> MessageResult<String> {
        Ok(std::str::from_utf8(self.bytes()?)?.to_string())
    }

    pub fn message<T: MessageDecode>(self) -> MessageResult<T> {
        T::decode(self.bytes()?)
    }
}

#[derive(Clone, Copy)]
enum FieldValue<'a> {
    Varint(u64),
    Fixed64(u64),
    Bytes(&'a [u8]),
    Fixed32(u32),
}

pub fn visit_fields(data: &[u8], mut visitor: impl FnMut(Field<'_>) -> MessageResult<()>) -> MessageResult<()> {
    let mut position = 0;
    while position < data.len() {
        let tag = read_varint(data, &mut position)?;
        let number = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u8;
        let value = match wire_type {
            WIRE_VARINT => FieldValue::Varint(read_varint(data, &mut position)?),
            WIRE_FIXED64 => FieldValue::Fixed64(read_fixed64(data, &mut position)?),
            WIRE_LENGTH_DELIMITED => {
                let len = read_varint(data, &mut position)? as usize;
                let end = position.checked_add(len).ok_or("invalid protobuf length")?;
                if end > data.len() {
                    return Err("truncated protobuf length-delimited field".into());
                }
                let value = &data[position..end];
                position = end;
                FieldValue::Bytes(value)
            }
            WIRE_FIXED32 => FieldValue::Fixed32(read_fixed32(data, &mut position)?),
            _ => return Err(format!("unsupported protobuf wire type {wire_type}").into()),
        };
        visitor(Field { number, value })?;
    }
    Ok(())
}

fn read_varint(data: &[u8], position: &mut usize) -> MessageResult<u64> {
    let mut result = 0u64;
    let mut shift = 0u32;
    while *position < data.len() {
        let byte = data[*position];
        *position += 1;
        let value = u64::from(byte & 0x7f);
        if shift == 63 && value > 1 {
            return Err("protobuf varint overflows u64".into());
        }
        result |= value << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err("protobuf varint is too long".into());
        }
    }
    Err("truncated protobuf varint".into())
}

fn read_fixed64(data: &[u8], position: &mut usize) -> MessageResult<u64> {
    let end = position.checked_add(8).ok_or("invalid protobuf fixed64 length")?;
    if end > data.len() {
        return Err("truncated protobuf fixed64".into());
    }
    let value = u64::from_le_bytes(data[*position..end].try_into()?);
    *position = end;
    Ok(value)
}

fn read_fixed32(data: &[u8], position: &mut usize) -> MessageResult<u32> {
    let end = position.checked_add(4).ok_or("invalid protobuf fixed32 length")?;
    if end > data.len() {
        return Err("truncated protobuf fixed32".into());
    }
    let value = u32::from_le_bytes(data[*position..end].try_into()?);
    *position = end;
    Ok(value)
}

#[macro_export]
macro_rules! proto_decode {
    ($type:ty { $($number:literal => $field_name:ident : $field_kind:ident),* $(,)? }) => {
        impl $crate::protobuf::MessageDecode for $type {
            fn decode(data: &[u8]) -> $crate::protobuf::MessageResult<Self> {
                let mut value = Self::default();
                $crate::protobuf::visit_fields(data, |field| {
                    match field.number {
                        $(
                            $number => {
                                $crate::protobuf::field_codec::$field_kind::decode(&mut value.$field_name, field)?;
                            }
                        )*
                        _ => {}
                    }
                    Ok(())
                })?;
                Ok(value)
            }
        }
    };
    ($type:ty { $($number:literal => |$value:ident, $field:ident| $body:expr),* $(,)? }) => {
        impl $crate::protobuf::MessageDecode for $type {
            fn decode(data: &[u8]) -> $crate::protobuf::MessageResult<Self> {
                let mut value = Self::default();
                $crate::protobuf::visit_fields(data, |field| {
                    match field.number {
                        $(
                            $number => {
                                (|$value: &mut Self, $field: $crate::protobuf::Field<'_>| -> $crate::protobuf::MessageResult<()> {
                                    $body;
                                    Ok(())
                                })(&mut value, field)?;
                            }
                        )*
                        _ => {}
                    }
                    Ok(())
                })?;
                Ok(value)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protobuf::{encode_raw_varint_field, encode_string_field};

    #[test]
    fn test_visit_fields() {
        let message = [encode_string_field(1, "test"), encode_raw_varint_field(2, 7)].concat();
        let mut values = Vec::new();

        visit_fields(&message, |field| {
            match field.number {
                1 => values.push(field.string()?),
                2 => values.push(field.varint()?.to_string()),
                _ => {}
            }
            Ok(())
        })
        .unwrap();

        assert_eq!(values, vec!["test", "7"]);
    }

    #[test]
    fn test_visit_fields_rejects_varint_overflow() {
        let message = [vec![0x08], vec![0xff; 9], vec![0x02]].concat();

        assert_eq!(visit_fields(&message, |_| Ok(())).unwrap_err().to_string(), "protobuf varint overflows u64");
    }

    #[derive(Debug, Default, PartialEq)]
    struct TestMessage {
        name: Option<String>,
        decimals: Option<u32>,
    }

    crate::proto_decode!(TestMessage {
        1 => name: optional_string,
        2 => decimals: optional_varint_u32,
    });

    #[test]
    fn test_proto_decode() {
        let data = [encode_string_field(1, "USDC"), encode_raw_varint_field(2, 6)].concat();

        assert_eq!(
            TestMessage::decode(&data).unwrap(),
            TestMessage {
                name: Some("USDC".into()),
                decimals: Some(6),
            }
        );
    }
}
