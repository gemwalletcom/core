use primitives::{
    ChainSigner,
    SignerError,
    TransactionInputType,
    TransactionLoadInput,
    TransactionLoadMetadata,
    TransferDataOutputAction,
    TransferDataOutputType,
    hex::decode_hex,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use signer::{SignatureScheme, Signer};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TronTransaction {
    raw_data_hex: Option<String>,
    #[serde(default)]
    signature: Vec<String>,
    #[serde(flatten)]
    other: Map<String, Value>,
}

struct PayloadMetadata {
    payload: Value,
    output_type: TransferDataOutputType,
    output_action: TransferDataOutputAction,
    raw_data_hex: String,
}

pub struct TronChainSigner;

impl ChainSigner for TronChainSigner {
    fn sign_data(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_data(input, private_key)
    }
}

fn sign_data(input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
    let (transaction, metadata) = extract_transaction(input)?;
    if !transaction.signature.is_empty() {
        return Err(invalid_input("Tron multisig not supported for WalletConnect signing"));
    }
    let raw_bytes = decode_hex(&metadata.raw_data_hex)?;
    let digest = Sha256::digest(&raw_bytes);
    let signature = sign_digest(&digest, private_key)?;
    let signature_hex = hex::encode(signature);

    match metadata.output_type {
        TransferDataOutputType::Signature => Ok(signature_hex),
        TransferDataOutputType::EncodedTransaction => {
            let payload = apply_signature(metadata.payload, &signature_hex)?;
            let result_payload = match metadata.output_action {
                TransferDataOutputAction::Send => extract_transaction_value(&payload)
                    .ok_or_else(|| invalid_input("Missing transaction object for Tron broadcast"))?,
                TransferDataOutputAction::Sign => payload,
            };

            Ok(serde_json::to_string(&result_payload)?)
        }
    }
}

fn extract_transaction(input: &TransactionLoadInput) -> Result<(TronTransaction, PayloadMetadata), SignerError> {
    let TransactionInputType::Generic(_, _, extra) = &input.input_type else {
        return Err(invalid_input("Expected generic transaction input"));
    };
    let data = extra.data.as_ref().ok_or_else(|| invalid_input("Missing transaction data"))?;

    let payload = match serde_json::from_slice::<Value>(data)? {
        Value::String(raw_json) => serde_json::from_str::<Value>(&raw_json).unwrap_or(Value::String(raw_json)),
        value => value,
    };

    let Value::Object(map) = &payload else {
        return Err(invalid_input("Invalid Tron transaction payload"));
    };
    let transaction_value = map.get("transaction").cloned().ok_or_else(|| invalid_input("Missing transaction field"))?;
    let transaction: TronTransaction = serde_json::from_value(transaction_value)?;
    let raw_data_hex = match &input.metadata {
        TransactionLoadMetadata::Tron {
            raw_data_hex: Some(raw_data_hex),
            ..
        } => raw_data_hex.clone(),
        _ => transaction
            .raw_data_hex
            .clone()
            .ok_or_else(|| invalid_input("Missing raw_data_hex in Tron transaction payload"))?,
    };

    let metadata = PayloadMetadata {
        payload,
        output_type: extra.output_type.clone(),
        output_action: extra.output_action.clone(),
        raw_data_hex,
    };
    Ok((transaction, metadata))
}

fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec())
        .map_err(|err| invalid_input(err.to_string()))
}

fn extract_transaction_value(payload: &Value) -> Option<Value> {
    match payload {
        Value::Object(map) => map.get("transaction").cloned(),
        _ => None,
    }
}

fn apply_signature(payload: Value, signature_hex: &str) -> Result<Value, SignerError> {
    let Value::Object(mut map) = payload else {
        return Err(invalid_input("Invalid Tron transaction payload"));
    };
    let transaction_value = map.get("transaction").cloned().ok_or_else(|| invalid_input("Missing transaction field"))?;
    let mut transaction: TronTransaction = serde_json::from_value(transaction_value)?;
    if !transaction.signature.is_empty() {
        return Err(invalid_input("Tron multisig not supported for WalletConnect signing"));
    }
    transaction.signature = vec![signature_hex.to_string()];
    map.insert("transaction".to_string(), serde_json::to_value(transaction)?);
    map.entry("signature".to_string())
        .or_insert(serde_json::Value::String(signature_hex.to_string()));
    Ok(Value::Object(map))
}

fn invalid_input(message: impl Into<String>) -> SignerError {
    SignerError::InvalidInput(message.into())
}
