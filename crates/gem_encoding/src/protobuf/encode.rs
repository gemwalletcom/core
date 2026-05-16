use super::message::MessageEncode;
use super::wire::{WIRE_LENGTH_DELIMITED, WIRE_VARINT};

pub fn encode_varint(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut value = value;

    while value >= 0x80 {
        buf.push((value as u8) | 0x80);
        value >>= 7;
    }

    buf.push(value as u8);
    buf
}

fn field_tag(field_number: u32, wire_type: u8) -> Vec<u8> {
    encode_varint(((field_number as u64) << 3) | wire_type as u64)
}

pub fn encode_varint_field(field_number: u32, value: u64) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }

    encode_raw_varint_field(field_number, value)
}

pub fn encode_raw_varint_field(field_number: u32, value: u64) -> Vec<u8> {
    [field_tag(field_number, WIRE_VARINT), encode_varint(value)].concat()
}

pub fn encode_bytes_field(field_number: u32, data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let tag = field_tag(field_number, WIRE_LENGTH_DELIMITED);
    let len = encode_varint(data.len() as u64);
    let mut buf = Vec::with_capacity(tag.len() + len.len() + data.len());
    buf.extend_from_slice(&tag);
    buf.extend_from_slice(&len);
    buf.extend_from_slice(data);
    buf
}

pub fn encode_string_field(field_number: u32, value: &str) -> Vec<u8> {
    encode_bytes_field(field_number, value.as_bytes())
}

pub fn encode_message_field(field_number: u32, message: &[u8]) -> Vec<u8> {
    if message.is_empty() {
        return Vec::new();
    }

    encode_bytes_field(field_number, message)
}

pub fn encode_optional_message_field(field_number: u32, value: Option<&impl MessageEncode>) -> Vec<u8> {
    value.map(|value| encode_message_field(field_number, &value.encode())).unwrap_or_default()
}

pub fn encode_optional_string_field(field_number: u32, value: Option<&str>) -> Vec<u8> {
    value.map(|value| encode_string_field(field_number, value)).unwrap_or_default()
}

pub fn encode_optional_bytes_field(field_number: u32, value: Option<&[u8]>) -> Vec<u8> {
    value.map(|value| encode_bytes_field(field_number, value)).unwrap_or_default()
}

pub fn encode_optional_u64_field(field_number: u32, value: Option<u64>) -> Vec<u8> {
    value.map(|value| encode_raw_varint_field(field_number, value)).unwrap_or_default()
}

pub fn encode_optional_bool_field(field_number: u32, value: Option<bool>) -> Vec<u8> {
    value.map(|value| encode_raw_varint_field(field_number, u64::from(value))).unwrap_or_default()
}

#[macro_export]
macro_rules! proto_encode {
    ($type:ty { $($number:literal => $field_name:ident : $field_kind:ident),* $(,)? }) => {
        impl $crate::protobuf::MessageEncode for $type {
            fn encode(&self) -> Vec<u8> {
                let value = self;
                let mut data = Vec::new();
                $(
                    data.extend($crate::protobuf::field_codec::$field_kind::encode($number, &value.$field_name));
                )*
                data
            }
        }
    };
    ($type:ty as $value:ident { $($field:expr),* $(,)? }) => {
        impl $crate::protobuf::MessageEncode for $type {
            fn encode(&self) -> Vec<u8> {
                let $value = self;
                let mut data = Vec::new();
                $(
                    data.extend($field);
                )*
                data
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint() {
        assert_eq!(encode_varint(0), vec![0]);
        assert_eq!(encode_varint(1), vec![1]);
        assert_eq!(encode_varint(127), vec![127]);
        assert_eq!(encode_varint(128), vec![0x80, 0x01]);
        assert_eq!(encode_varint(300), vec![0xAC, 0x02]);
    }

    #[test]
    fn test_encode_string_field() {
        let result = encode_string_field(1, "test");

        assert_eq!(result, vec![0x0A, 4, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_empty_fields_omitted() {
        assert!(encode_varint_field(1, 0).is_empty());
        assert!(encode_string_field(1, "").is_empty());
        assert!(encode_bytes_field(1, &[]).is_empty());
    }

    #[test]
    fn test_encode_optional_u64_field_keeps_present_zero() {
        assert_eq!(encode_optional_u64_field(1, Some(0)), vec![0x08, 0x00]);
        assert!(encode_optional_u64_field(1, None).is_empty());
    }

    #[derive(Debug, Default)]
    struct TestMessage {
        name: Option<String>,
        decimals: Option<u32>,
    }

    crate::proto_encode!(TestMessage {
        1 => name: optional_string,
        2 => decimals: optional_varint_u32,
    });

    #[test]
    fn test_proto_encode() {
        let message = TestMessage {
            name: Some("USDC".into()),
            decimals: Some(6),
        };

        assert_eq!(message.encode(), [encode_string_field(1, "USDC"), encode_raw_varint_field(2, 6)].concat());
    }
}
