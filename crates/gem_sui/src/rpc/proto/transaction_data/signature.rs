use gem_encoding::protobuf::proto_encode;
use sui_types as sdk;

use crate::rpc::proto::Bcs;

// Field numbers mirror sui-rpc v0.3.1 signature schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/signature.proto

#[derive(Clone, Debug, Default)]
pub struct UserSignature {
    pub bcs: Option<Bcs>,
    pub scheme: Option<i32>,
}

impl UserSignature {
    pub fn from_sdk(signature: sdk::UserSignature) -> Self {
        Self {
            bcs: Some(Bcs::new("UserSignatureBytes", signature.to_bytes())),
            scheme: Some(signature.scheme().to_u8().into()),
        }
    }
}

proto_encode!(UserSignature {
    1 => bcs: optional_message,
    2 => scheme: optional_varint_i32,
});
