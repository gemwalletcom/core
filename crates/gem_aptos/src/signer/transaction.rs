use crate::models::{SubmitTransactionRequest, TransactionSignature};
use primitives::SignerError;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::time::SystemTime;

use super::{AccountAddress, EntryFunction, EntryFunctionPayload};

const RAW_TRANSACTION_SALT: &[u8] = b"APTOS::RawTransaction";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    pub code: Vec<u8>,
    pub ty_args: Vec<super::TypeTag>,
    pub args: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeprecatedPayload {
    pub modules: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionPayloadBCS {
    Script(Script),
    ModuleBundle(DeprecatedPayload),
    EntryFunction(EntryFunction),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTransaction {
    pub sender: AccountAddress,
    pub sequence_number: u64,
    pub payload: TransactionPayloadBCS,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
    pub chain_id: u8,
}

pub fn build_raw_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    payload: EntryFunction,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    chain_id: u8,
) -> RawTransaction {
    RawTransaction {
        sender,
        sequence_number,
        payload: TransactionPayloadBCS::EntryFunction(payload),
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        chain_id,
    }
}

pub fn sign_raw_transaction(raw_tx: &RawTransaction, private_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SignerError> {
    let raw_tx_bytes = bcs::to_bytes(raw_tx)
        .map_err(|err| SignerError::InvalidInput(format!("Failed to encode Aptos transaction: {err}")))?;
    let seed = sha3_256(RAW_TRANSACTION_SALT);
    let mut preimage = Vec::with_capacity(seed.len() + raw_tx_bytes.len());
    preimage.extend_from_slice(&seed);
    preimage.extend_from_slice(&raw_tx_bytes);
    let digest = sha3_256(&preimage);

    signer::Signer::sign_ed25519_with_public_key(&digest, private_key)
        .map_err(|err| SignerError::InvalidInput(err.to_string()))
}

pub fn build_submit_transaction(
    raw_tx: RawTransaction,
    payload: &EntryFunctionPayload,
    signature: Vec<u8>,
    public_key: Vec<u8>,
) -> Result<String, SignerError> {
    let signature = TransactionSignature {
        signature_type: "ed25519_signature".to_string(),
        public_key: Some(format_hex(&public_key)),
        signature: Some(format_hex(&signature)),
    };

    let request = SubmitTransactionRequest {
        sender: raw_tx.sender.to_hex(),
        sequence_number: raw_tx.sequence_number.to_string(),
        max_gas_amount: raw_tx.max_gas_amount.to_string(),
        gas_unit_price: raw_tx.gas_unit_price.to_string(),
        expiration_timestamp_secs: raw_tx.expiration_timestamp_secs.to_string(),
        payload: payload.to_transaction_payload(),
        signature,
    };

    serde_json::to_string(&request).map_err(|err| SignerError::InvalidInput(err.to_string()))
}

pub fn expiration_timestamp_secs() -> Result<u64, SignerError> {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| SignerError::InvalidInput("Invalid system time".to_string()))?;
    Ok(now.as_secs() + 3_600)
}

fn sha3_256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

fn format_hex(value: &[u8]) -> String {
    format!("0x{}", hex::encode(value))
}
