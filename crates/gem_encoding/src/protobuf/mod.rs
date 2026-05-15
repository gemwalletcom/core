mod decode;
mod encode;
pub mod field_codec;
mod grpc;
mod message;
mod wire;

pub use crate::{proto_decode, proto_encode};
pub use decode::{Field, visit_fields};
pub use encode::{
    encode_bytes_field, encode_message_field, encode_optional_bool_field, encode_optional_bytes_field, encode_optional_message_field, encode_optional_string_field,
    encode_optional_u64_field, encode_raw_varint_field, encode_string_field, encode_varint, encode_varint_field,
};
pub use grpc::{decode_grpc_frame, decode_grpc_message, encode_grpc_frame, encode_grpc_message};
pub use message::{Message, MessageDecode, MessageEncode, MessageResult};
