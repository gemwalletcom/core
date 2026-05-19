use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBuilderJson {
    pub version: u8,
    pub inputs: Vec<TransactionInput>,
    pub commands: Vec<TransactionCommand>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TransactionInput {
    Pure {
        #[serde(rename = "Pure")]
        pure: PureInput,
    },
    Object {
        #[serde(rename = "Object")]
        object: TransactionObject,
    },
    UnresolvedObject {
        #[serde(rename = "UnresolvedObject")]
        object: UnresolvedObject,
    },
    UnresolvedPure {
        #[serde(rename = "UnresolvedPure")]
        pure: Value,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct PureInput {
    pub bytes: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedObject {
    pub object_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TransactionObject {
    ImmOrOwnedObject {
        #[serde(rename = "ImmOrOwnedObject")]
        object: ObjectRef,
    },
    SharedObject {
        #[serde(rename = "SharedObject")]
        object: SharedObjectRef,
    },
    Receiving {
        #[serde(rename = "Receiving")]
        object: ObjectRef,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRef {
    pub object_id: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub version: u64,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedObjectRef {
    pub object_id: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub initial_shared_version: u64,
    pub mutable: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TransactionCommand {
    MoveCall {
        #[serde(rename = "MoveCall")]
        move_call: MoveCallCommand,
    },
    TransferObjects {
        #[serde(rename = "TransferObjects")]
        transfer_objects: TransferObjectsCommand,
    },
    SplitCoins {
        #[serde(rename = "SplitCoins")]
        split_coins: SplitCoinsCommand,
    },
    MergeCoins {
        #[serde(rename = "MergeCoins")]
        merge_coins: MergeCoinsCommand,
    },
    MakeMoveVec {
        #[serde(rename = "MakeMoveVec")]
        make_move_vec: MakeMoveVecCommand,
    },
    Publish {
        #[serde(rename = "Publish")]
        publish: Value,
    },
    Upgrade {
        #[serde(rename = "Upgrade")]
        upgrade: Value,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCallCommand {
    pub package: String,
    pub module: String,
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<TransactionArgument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransferObjectsCommand {
    pub objects: Vec<TransactionArgument>,
    pub address: TransactionArgument,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SplitCoinsCommand {
    pub coin: TransactionArgument,
    pub amounts: Vec<TransactionArgument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MergeCoinsCommand {
    pub destination: TransactionArgument,
    pub sources: Vec<TransactionArgument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MakeMoveVecCommand {
    pub r#type: Option<String>,
    pub elements: Vec<TransactionArgument>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TransactionArgument {
    GasCoin {
        #[serde(rename = "GasCoin")]
        gas_coin: bool,
    },
    Input {
        #[serde(rename = "Input")]
        input: usize,
    },
    Result {
        #[serde(rename = "Result")]
        result: usize,
    },
    NestedResult {
        #[serde(rename = "NestedResult")]
        nested_result: [usize; 2],
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transaction_json() {
        let transaction: TransactionBuilderJson = serde_json::from_str(include_str!("../../../testdata/transaction_builder_json.json")).unwrap();

        assert_eq!(transaction.version, 2);
        assert_eq!(transaction.inputs.len(), 2);
        assert_eq!(transaction.commands.len(), 2);
    }
}
