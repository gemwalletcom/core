use gem_encoding::protobuf::proto_encode;
use sui_types as sdk;

// Field numbers mirror sui-rpc v0.3.1 argument schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/argument.proto

const ARGUMENT_KIND_GAS: i32 = 1;
const ARGUMENT_KIND_INPUT: i32 = 2;
const ARGUMENT_KIND_RESULT: i32 = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Argument {
    pub kind: Option<i32>,
    pub input: Option<u32>,
    pub result: Option<u32>,
    pub subresult: Option<u32>,
}

impl Argument {
    pub fn new_input(input: u16) -> Self {
        Self {
            kind: Some(ARGUMENT_KIND_INPUT),
            input: Some(input.into()),
            ..Default::default()
        }
    }

    pub(super) fn from_sdk(value: sdk::Argument) -> Self {
        match value {
            sdk::Argument::Gas => Self {
                kind: Some(ARGUMENT_KIND_GAS),
                ..Default::default()
            },
            sdk::Argument::Input(input) => Self::new_input(input),
            sdk::Argument::Result(result) => Self {
                kind: Some(ARGUMENT_KIND_RESULT),
                result: Some(result.into()),
                ..Default::default()
            },
            sdk::Argument::NestedResult(result, subresult) => Self {
                kind: Some(ARGUMENT_KIND_RESULT),
                result: Some(result.into()),
                subresult: Some(subresult.into()),
                ..Default::default()
            },
        }
    }
}

proto_encode!(Argument {
    1 => kind: optional_varint_i32,
    2 => input: optional_varint_u32,
    3 => result: optional_varint_u32,
    4 => subresult: optional_varint_u32,
});
