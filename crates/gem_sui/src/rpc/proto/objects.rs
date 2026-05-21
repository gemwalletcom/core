use sui_types::Address as SdkAddress;

use serde_json::Value;

use super::json::decode_json_value;
use super::{Bcs, FieldMask, Status};
use gem_encoding::protobuf::{proto_decode, proto_encode};

// Field numbers mirror sui-rpc v0.3.1 ledger/object schemas:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/ledger_service.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/object.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/owner.proto

#[derive(Clone, Debug, Default)]
pub struct ListOwnedObjectsRequest {
    pub owner: Option<String>,
    pub page_size: Option<u32>,
    pub page_token: Option<Vec<u8>>,
    pub read_mask: Option<FieldMask>,
    pub object_type: Option<String>,
}

proto_encode!(ListOwnedObjectsRequest {
    1 => owner: optional_string,
    2 => page_size: optional_varint_u32,
    3 => page_token: optional_bytes,
    4 => read_mask: optional_message,
    5 => object_type: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct ListOwnedObjectsResponse {
    pub objects: Vec<Object>,
    pub next_page_token: Option<Vec<u8>>,
}

proto_decode!(ListOwnedObjectsResponse {
    1 => objects: repeated_message,
    2 => next_page_token: optional_bytes,
});

#[derive(Clone, Debug, Default)]
pub struct GetObjectRequest {
    pub object_id: Option<String>,
    pub read_mask: Option<FieldMask>,
}

impl GetObjectRequest {
    pub fn new(object_id: &SdkAddress) -> Self {
        Self {
            object_id: Some(object_id.to_string()),
            read_mask: None,
        }
    }
}

proto_encode!(GetObjectRequest {
    1 => object_id: optional_string,
    3 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct GetObjectResponse {
    pub object: Option<Object>,
}

proto_decode!(GetObjectResponse {
    1 => object: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct BatchGetObjectsRequest {
    pub requests: Vec<GetObjectRequest>,
    pub read_mask: Option<FieldMask>,
}

proto_encode!(BatchGetObjectsRequest {
    1 => requests: repeated_message,
    2 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct BatchGetObjectsResponse {
    pub objects: Vec<GetObjectResult>,
}

proto_decode!(BatchGetObjectsResponse {
    1 => objects: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub enum GetObjectResult {
    Object(Box<Object>),
    Error(Status),
    #[default]
    Unknown,
}

proto_decode!(GetObjectResult {
    1 => |value, field| *value = Self::Object(Box::new(field.message()?)),
    2 => |value, field| *value = Self::Error(field.message()?),
});

#[derive(Clone, Debug, Default)]
pub struct Object {
    pub object_id: Option<String>,
    pub version: Option<u64>,
    pub digest: Option<String>,
    pub owner: Option<Owner>,
    pub object_type: Option<String>,
    pub contents: Option<Bcs>,
    pub json: Option<Value>,
    pub balance: Option<u64>,
}

proto_decode!(Object {
    2 => |value, field| value.object_id = Some(field.string()?),
    3 => |value, field| value.version = Some(field.varint()?),
    4 => |value, field| value.digest = Some(field.string()?),
    5 => |value, field| value.owner = Some(field.message()?),
    6 => |value, field| value.object_type = Some(field.string()?),
    8 => |value, field| value.contents = Some(field.message()?),
    100 => |value, field| value.json = Some(decode_json_value(field.bytes()?)?),
    101 => |value, field| value.balance = Some(field.varint()?),
});

#[derive(Clone, Debug, Default)]
pub struct Owner {
    pub kind: Option<i32>,
    pub address: Option<String>,
    pub version: Option<u64>,
}

impl Owner {
    pub fn kind(&self) -> OwnerKind {
        OwnerKind::from_i32(self.kind.unwrap_or_default())
    }
}

proto_decode!(Owner {
    1 => kind: optional_varint_i32,
    2 => address: optional_string,
    3 => version: optional_varint_u64,
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerKind {
    Unknown,
    Address,
    Object,
    Shared,
    Immutable,
    ConsensusAddress,
}

impl OwnerKind {
    fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Address,
            2 => Self::Object,
            3 => Self::Shared,
            4 => Self::Immutable,
            5 => Self::ConsensusAddress,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::MessageDecode;

    #[test]
    fn test_object_response_wire_bytes_decode() {
        let response = hex::decode("0a2b120530786162631807220a6f626a2d6469676573742a0b0801120730786f776e65723204636f696ea8067b").unwrap();
        let object = GetObjectResponse::decode(&response).unwrap().object.unwrap();
        let owner = object.owner.unwrap();

        assert_eq!(object.object_id.as_deref(), Some("0xabc"));
        assert_eq!(object.version, Some(7));
        assert_eq!(object.digest.as_deref(), Some("obj-digest"));
        assert_eq!(owner.kind(), OwnerKind::Address);
        assert_eq!(owner.address.as_deref(), Some("0xowner"));
        assert_eq!(object.object_type.as_deref(), Some("coin"));
        assert_eq!(object.balance, Some(123));
    }
}
