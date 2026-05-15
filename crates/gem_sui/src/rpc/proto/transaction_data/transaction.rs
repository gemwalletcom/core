use gem_encoding::protobuf::proto_encode;
use sui_types as sdk;

use super::{Command, Input};
use crate::rpc::proto::{Bcs, MessageResult};

// Field numbers mirror sui-rpc v0.3.1 transaction data schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/transaction.proto

const KIND_PROGRAMMABLE_TRANSACTION: i32 = 1;
const KIND_PROGRAMMABLE_SYSTEM_TRANSACTION: i32 = 11;

#[derive(Clone, Debug, Default)]
pub struct Transaction {
    pub bcs: Option<Bcs>,
    pub kind: Option<TransactionKind>,
    pub sender: Option<String>,
}

impl Transaction {
    pub fn from_transaction_bcs(value: Vec<u8>) -> Self {
        Self {
            bcs: Some(Bcs::new("TransactionData", value)),
            ..Default::default()
        }
    }

    pub fn from_kind(kind: TransactionKind, sender: &str) -> Self {
        Self {
            kind: Some(kind),
            sender: Some(sender.to_string()),
            ..Default::default()
        }
    }
}

proto_encode!(Transaction {
    1 => bcs: optional_message,
    4 => kind: optional_message,
    5 => sender: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct TransactionKind {
    pub kind: Option<i32>,
    pub programmable_transaction: Option<ProgrammableTransaction>,
}

impl TransactionKind {
    pub fn programmable_transaction(transaction: ProgrammableTransaction) -> Self {
        Self {
            kind: Some(KIND_PROGRAMMABLE_TRANSACTION),
            programmable_transaction: Some(transaction),
        }
    }

    pub fn from_sdk(value: sdk::TransactionKind) -> MessageResult<Self> {
        match value {
            sdk::TransactionKind::ProgrammableTransaction(transaction) => Ok(Self {
                kind: Some(KIND_PROGRAMMABLE_TRANSACTION),
                programmable_transaction: Some(ProgrammableTransaction::from_sdk(transaction)?),
            }),
            sdk::TransactionKind::ProgrammableSystemTransaction(transaction) => Ok(Self {
                kind: Some(KIND_PROGRAMMABLE_SYSTEM_TRANSACTION),
                programmable_transaction: Some(ProgrammableTransaction::from_sdk(transaction)?),
            }),
            _ => Err("unsupported Sui transaction kind for protobuf encoding".into()),
        }
    }
}

proto_encode!(TransactionKind {
    1 => kind: optional_varint_i32,
    2 => programmable_transaction: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ProgrammableTransaction {
    pub inputs: Vec<Input>,
    pub commands: Vec<Command>,
}

impl ProgrammableTransaction {
    pub fn from_sdk(value: sdk::ProgrammableTransaction) -> MessageResult<Self> {
        Ok(Self {
            inputs: value.inputs.into_iter().map(Input::from_sdk).collect::<MessageResult<_>>()?,
            commands: value.commands.into_iter().map(Command::from_sdk).collect::<MessageResult<_>>()?,
        })
    }
}

proto_encode!(ProgrammableTransaction {
    1 => inputs: repeated_message,
    2 => commands: repeated_message,
});
