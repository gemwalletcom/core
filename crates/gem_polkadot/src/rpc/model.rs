use core::str;

use serde::{Deserialize, Serialize};

pub const TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE: &str = "transferKeepAlive";
pub const TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH: &str = "transferAllowDeath";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: String,
    pub extrinsics: Vec<Extrinsic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extrinsic {
    pub hash: String,
    pub nonce: Option<String>,
    pub method: ExtrinsicMethod,
    pub info: ExtrinsicInfo,
    pub success: bool,
    pub args: ExtrinsicArguments,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicMethod {
    pub pallet: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtrinsicInfo {
    pub partial_fee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtrinsicArguments {
    Transfer(ExtrinsicTransfer),
    Transfers(ExtrinsicCalls),
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicTransfer {
    pub value: String,
    pub dest: AddressId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicCalls {
    pub calls: Vec<Call>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub method: ExtrinsicMethod,
    pub args: CallArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallArgs {
    pub dest: AddressId,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressId {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signer: AddressId,
}
