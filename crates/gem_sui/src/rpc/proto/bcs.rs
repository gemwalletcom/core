use gem_encoding::protobuf::{proto_decode, proto_encode};
use serde::Deserialize;

// Field numbers mirror sui-rpc v0.3.1 BCS protobuf schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/bcs.proto

#[derive(Clone, Debug, Default)]
pub struct Bcs {
    pub name: Option<String>,
    pub value: Option<Vec<u8>>,
}

impl Bcs {
    pub fn new(name: impl Into<String>, value: Vec<u8>) -> Self {
        Self {
            name: Some(name.into()),
            value: Some(value),
        }
    }

    pub fn deserialize<'de, T: Deserialize<'de>>(&'de self) -> Result<T, ::bcs::Error> {
        ::bcs::from_bytes(self.value.as_deref().unwrap_or(&[]))
    }
}

proto_encode!(Bcs {
    1 => name: optional_string,
    2 => value: optional_bytes,
});

proto_decode!(Bcs {
    1 => name: optional_string,
    2 => value: optional_bytes,
});
