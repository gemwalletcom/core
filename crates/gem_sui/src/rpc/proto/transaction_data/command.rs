use gem_encoding::protobuf::{encode_message_field, proto_encode};
use sui_types as sdk;

use super::Argument;
use crate::rpc::proto::MessageResult;

// Field numbers mirror sui-rpc v0.3.1 transaction command schemas:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/transaction.proto

#[derive(Clone, Debug)]
pub enum Command {
    MoveCall(MoveCall),
    TransferObjects(TransferObjects),
    SplitCoins(SplitCoins),
    MergeCoins(MergeCoins),
    Publish(Publish),
    MakeMoveVector(MakeMoveVector),
    Upgrade(Upgrade),
}

impl Command {
    pub(super) fn from_sdk(value: sdk::Command) -> MessageResult<Self> {
        match value {
            sdk::Command::MoveCall(value) => Ok(Self::MoveCall(MoveCall::from_sdk(value))),
            sdk::Command::TransferObjects(value) => Ok(Self::TransferObjects(TransferObjects::from_sdk(value))),
            sdk::Command::SplitCoins(value) => Ok(Self::SplitCoins(SplitCoins::from_sdk(value))),
            sdk::Command::MergeCoins(value) => Ok(Self::MergeCoins(MergeCoins::from_sdk(value))),
            sdk::Command::Publish(value) => Ok(Self::Publish(Publish::from_sdk(value))),
            sdk::Command::MakeMoveVector(value) => Ok(Self::MakeMoveVector(MakeMoveVector::from_sdk(value))),
            sdk::Command::Upgrade(value) => Ok(Self::Upgrade(Upgrade::from_sdk(value))),
            _ => Err("unsupported Sui transaction command for protobuf encoding".into()),
        }
    }
}

impl From<MoveCall> for Command {
    fn from(value: MoveCall) -> Self {
        Self::MoveCall(value)
    }
}

proto_encode!(Command as value {
    match value {
        Command::MoveCall(value) => encode_message_field(1, &value.encode()),
        Command::TransferObjects(value) => encode_message_field(2, &value.encode()),
        Command::SplitCoins(value) => encode_message_field(3, &value.encode()),
        Command::MergeCoins(value) => encode_message_field(4, &value.encode()),
        Command::Publish(value) => encode_message_field(5, &value.encode()),
        Command::MakeMoveVector(value) => encode_message_field(6, &value.encode()),
        Command::Upgrade(value) => encode_message_field(7, &value.encode()),
    },
});

#[derive(Clone, Debug, Default)]
pub struct MoveCall {
    pub package: Option<String>,
    pub module: Option<String>,
    pub function: Option<String>,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<Argument>,
}

impl MoveCall {
    pub fn from_parts(package: impl ToString, module: impl Into<String>, function: impl Into<String>, arguments: Vec<Argument>) -> Self {
        Self {
            package: Some(package.to_string()),
            module: Some(module.into()),
            function: Some(function.into()),
            arguments,
            ..Default::default()
        }
    }

    fn from_sdk(value: sdk::MoveCall) -> Self {
        Self {
            package: Some(value.package.to_string()),
            module: Some(value.module.to_string()),
            function: Some(value.function.to_string()),
            type_arguments: value.type_arguments.iter().map(ToString::to_string).collect(),
            arguments: value.arguments.into_iter().map(Argument::from_sdk).collect(),
        }
    }
}

proto_encode!(MoveCall {
    1 => package: optional_string,
    2 => module: optional_string,
    3 => function: optional_string,
    4 => type_arguments: repeated_string,
    5 => arguments: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct TransferObjects {
    pub objects: Vec<Argument>,
    pub address: Option<Argument>,
}

impl TransferObjects {
    fn from_sdk(value: sdk::TransferObjects) -> Self {
        Self {
            objects: value.objects.into_iter().map(Argument::from_sdk).collect(),
            address: Some(Argument::from_sdk(value.address)),
        }
    }
}

proto_encode!(TransferObjects {
    1 => objects: repeated_message,
    2 => address: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct SplitCoins {
    pub coin: Option<Argument>,
    pub amounts: Vec<Argument>,
}

impl SplitCoins {
    fn from_sdk(value: sdk::SplitCoins) -> Self {
        Self {
            coin: Some(Argument::from_sdk(value.coin)),
            amounts: value.amounts.into_iter().map(Argument::from_sdk).collect(),
        }
    }
}

proto_encode!(SplitCoins {
    1 => coin: optional_message,
    2 => amounts: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct MergeCoins {
    pub coin: Option<Argument>,
    pub coins_to_merge: Vec<Argument>,
}

impl MergeCoins {
    fn from_sdk(value: sdk::MergeCoins) -> Self {
        Self {
            coin: Some(Argument::from_sdk(value.coin)),
            coins_to_merge: value.coins_to_merge.into_iter().map(Argument::from_sdk).collect(),
        }
    }
}

proto_encode!(MergeCoins {
    1 => coin: optional_message,
    2 => coins_to_merge: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct Publish {
    pub modules: Vec<Vec<u8>>,
    pub dependencies: Vec<String>,
}

impl Publish {
    fn from_sdk(value: sdk::Publish) -> Self {
        Self {
            modules: value.modules,
            dependencies: value.dependencies.iter().map(ToString::to_string).collect(),
        }
    }
}

proto_encode!(Publish {
    1 => modules: repeated_bytes,
    2 => dependencies: repeated_string,
});

#[derive(Clone, Debug, Default)]
pub struct MakeMoveVector {
    pub element_type: Option<String>,
    pub elements: Vec<Argument>,
}

impl MakeMoveVector {
    fn from_sdk(value: sdk::MakeMoveVector) -> Self {
        Self {
            element_type: value.type_.map(|value| value.to_string()),
            elements: value.elements.into_iter().map(Argument::from_sdk).collect(),
        }
    }
}

proto_encode!(MakeMoveVector {
    1 => element_type: optional_string,
    2 => elements: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct Upgrade {
    pub modules: Vec<Vec<u8>>,
    pub dependencies: Vec<String>,
    pub package: Option<String>,
    pub ticket: Option<Argument>,
}

impl Upgrade {
    fn from_sdk(value: sdk::Upgrade) -> Self {
        Self {
            modules: value.modules,
            dependencies: value.dependencies.iter().map(ToString::to_string).collect(),
            package: Some(value.package.to_string()),
            ticket: Some(Argument::from_sdk(value.ticket)),
        }
    }
}

proto_encode!(Upgrade {
    1 => modules: repeated_bytes,
    2 => dependencies: repeated_string,
    3 => package: optional_string,
    4 => ticket: optional_message,
});
