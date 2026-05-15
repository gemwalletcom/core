use serde_json::{Map, Number, Value};

use super::MessageResult;
use gem_encoding::protobuf::{MessageDecode, proto_decode};

// Field numbers mirror google.protobuf.Value, Struct, and ListValue:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/google/protobuf/struct.proto

pub(super) fn decode_json_value(data: &[u8]) -> MessageResult<Value> {
    Ok(JsonValue::decode(data)?.into_value())
}

#[derive(Clone, Debug)]
struct JsonValue {
    value: Value,
}

impl Default for JsonValue {
    fn default() -> Self {
        Self { value: Value::Null }
    }
}

impl JsonValue {
    fn into_value(self) -> Value {
        self.value
    }
}

proto_decode!(JsonValue {
    1 => |value, _field| value.value = Value::Null,
    2 => |value, field| value.value = Number::from_f64(f64::from_bits(field.fixed64()?)).map(Value::Number).unwrap_or(Value::Null),
    3 => |value, field| value.value = Value::String(field.string()?),
    4 => |value, field| value.value = Value::Bool(field.varint()? != 0),
    5 => |value, field| value.value = field.message::<JsonStruct>()?.into_value(),
    6 => |value, field| value.value = field.message::<JsonList>()?.into_value(),
});

#[derive(Clone, Debug, Default)]
struct JsonStruct {
    fields: Map<String, Value>,
}

impl JsonStruct {
    fn into_value(self) -> Value {
        Value::Object(self.fields)
    }
}

proto_decode!(JsonStruct {
    1 => |value, field| {
        let entry = field.message::<JsonStructField>()?;
        value.fields.insert(entry.key, entry.value);
    },
});

#[derive(Clone, Debug)]
struct JsonStructField {
    key: String,
    value: Value,
}

impl Default for JsonStructField {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: Value::Null,
        }
    }
}

proto_decode!(JsonStructField {
    1 => |value, field| value.key = field.string()?,
    2 => |value, field| value.value = field.message::<JsonValue>()?.into_value(),
});

#[derive(Clone, Debug, Default)]
struct JsonList {
    values: Vec<Value>,
}

impl JsonList {
    fn into_value(self) -> Value {
        Value::Array(self.values)
    }
}

proto_decode!(JsonList {
    1 => |value, field| value.values.push(field.message::<JsonValue>()?.into_value()),
});

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::{encode_message_field, encode_raw_varint_field, encode_string_field};
    use serde_json::json;

    #[test]
    fn test_decode_json_value() {
        let name = [encode_string_field(1, "name"), encode_message_field(2, &encode_string_field(3, "Sui"))].concat();
        let active = [encode_string_field(1, "active"), encode_message_field(2, &encode_raw_varint_field(4, 1))].concat();
        let list = [
            encode_message_field(1, &encode_string_field(3, "rpc")),
            encode_message_field(1, &encode_raw_varint_field(4, 1)),
        ]
        .concat();
        let tags = [encode_string_field(1, "tags"), encode_message_field(2, &encode_message_field(6, &list))].concat();
        let object = [encode_message_field(1, &name), encode_message_field(1, &active), encode_message_field(1, &tags)].concat();
        let value = encode_message_field(5, &object);

        assert_eq!(
            decode_json_value(&value).unwrap(),
            json!({
                "name": "Sui",
                "active": true,
                "tags": ["rpc", true],
            })
        );
    }
}
