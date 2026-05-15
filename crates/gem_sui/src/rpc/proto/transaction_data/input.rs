use gem_encoding::protobuf::proto_encode;
use sui_types as sdk;

use crate::rpc::proto::MessageResult;

// Field numbers mirror sui-rpc v0.3.1 input schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/input.proto

const INPUT_KIND_PURE: i32 = 1;
const INPUT_KIND_IMMUTABLE_OR_OWNED: i32 = 2;
const INPUT_KIND_SHARED: i32 = 3;
const INPUT_KIND_RECEIVING: i32 = 4;
const INPUT_KIND_FUNDS_WITHDRAWAL: i32 = 5;

const MUTABILITY_IMMUTABLE: i32 = 1;
const MUTABILITY_MUTABLE: i32 = 2;
const MUTABILITY_NON_EXCLUSIVE_WRITE: i32 = 3;

const SOURCE_SENDER: i32 = 1;
const SOURCE_SPONSOR: i32 = 2;

#[derive(Clone, Debug, Default)]
pub struct Input {
    pub kind: Option<i32>,
    pub pure: Option<Vec<u8>>,
    pub object_id: Option<String>,
    pub version: Option<u64>,
    pub digest: Option<String>,
    pub mutable: Option<bool>,
    pub mutability: Option<i32>,
    pub funds_withdrawal: Option<FundsWithdrawal>,
}

impl Input {
    pub fn object_id(object_id: impl ToString) -> Self {
        Self {
            object_id: Some(object_id.to_string()),
            ..Default::default()
        }
    }

    pub fn pure(value: Vec<u8>) -> Self {
        Self {
            pure: Some(value),
            ..Default::default()
        }
    }

    pub(super) fn from_sdk(value: sdk::Input) -> MessageResult<Self> {
        match value {
            sdk::Input::Pure(value) => Ok(Self {
                kind: Some(INPUT_KIND_PURE),
                pure: Some(value),
                ..Default::default()
            }),
            sdk::Input::ImmutableOrOwned(reference) => Ok(Self {
                kind: Some(INPUT_KIND_IMMUTABLE_OR_OWNED),
                object_id: Some(reference.object_id().to_string()),
                version: Some(reference.version()),
                digest: Some(reference.digest().to_string()),
                ..Default::default()
            }),
            sdk::Input::Shared(shared) => {
                let mutability = match shared.mutability() {
                    sdk::Mutability::Immutable => MUTABILITY_IMMUTABLE,
                    sdk::Mutability::Mutable => MUTABILITY_MUTABLE,
                    sdk::Mutability::NonExclusiveWrite => MUTABILITY_NON_EXCLUSIVE_WRITE,
                };
                Ok(Self {
                    kind: Some(INPUT_KIND_SHARED),
                    object_id: Some(shared.object_id().to_string()),
                    version: Some(shared.version()),
                    mutable: Some(shared.mutability().is_mutable()),
                    mutability: Some(mutability),
                    ..Default::default()
                })
            }
            sdk::Input::Receiving(reference) => Ok(Self {
                kind: Some(INPUT_KIND_RECEIVING),
                object_id: Some(reference.object_id().to_string()),
                version: Some(reference.version()),
                digest: Some(reference.digest().to_string()),
                ..Default::default()
            }),
            sdk::Input::FundsWithdrawal(withdrawal) => Ok(Self {
                kind: Some(INPUT_KIND_FUNDS_WITHDRAWAL),
                funds_withdrawal: Some(FundsWithdrawal::from_sdk(withdrawal)),
                ..Default::default()
            }),
            _ => Err("unsupported Sui transaction input for protobuf encoding".into()),
        }
    }
}

proto_encode!(Input {
    1 => kind: optional_varint_i32,
    2 => pure: optional_bytes,
    3 => object_id: optional_string,
    4 => version: optional_varint_u64,
    5 => digest: optional_string,
    6 => mutable: optional_bool,
    7 => mutability: optional_varint_i32,
    8 => funds_withdrawal: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct FundsWithdrawal {
    pub amount: Option<u64>,
    pub coin_type: Option<String>,
    pub source: Option<i32>,
}

impl FundsWithdrawal {
    fn from_sdk(value: sdk::FundsWithdrawal) -> Self {
        let source = match value.source() {
            sdk::WithdrawFrom::Sender => SOURCE_SENDER,
            sdk::WithdrawFrom::Sponsor => SOURCE_SPONSOR,
            _ => 0,
        };
        Self {
            amount: value.amount(),
            coin_type: Some(value.coin_type().to_string()),
            source: Some(source),
        }
    }
}

proto_encode!(FundsWithdrawal {
    1 => amount: optional_varint_u64,
    2 => coin_type: optional_string,
    3 => source: optional_varint_i32,
});
