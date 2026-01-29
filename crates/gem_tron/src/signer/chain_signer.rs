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
        return Err(SignerError::InvalidInput("Tron multisig not supported for WalletConnect signing".to_string()));
    }
    let raw_bytes = decode_hex(&metadata.raw_data_hex)?;
    let digest = Sha256::digest(&raw_bytes);
    let signature = Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec())
        .map_err(|err| SignerError::InvalidInput(err.to_string()))?;
    let signature_hex = hex::encode(signature);

    match metadata.output_type {
        TransferDataOutputType::Signature => Ok(signature_hex),
        TransferDataOutputType::EncodedTransaction => {
            let payload = apply_signature(metadata.payload, &signature_hex)?;
            let result_payload = match metadata.output_action {
                TransferDataOutputAction::Send => {
                    extract_transaction_payload(&payload)
                        .ok_or_else(|| SignerError::InvalidInput("Missing transaction object for Tron broadcast".to_string()))?
                }
                TransferDataOutputAction::Sign => payload,
            };

            Ok(serde_json::to_string(&result_payload)?)
        }
    }
}

fn extract_transaction(input: &TransactionLoadInput) -> Result<(TronTransaction, PayloadMetadata), SignerError> {
    let TransactionInputType::Generic(_, _, extra) = &input.input_type else {
        return Err(SignerError::InvalidInput("Expected generic transaction input".to_string()));
    };
    let data = extra.data.as_ref().ok_or_else(|| SignerError::InvalidInput("Missing transaction data".to_string()))?;

    let mut payload: Value = serde_json::from_slice(data)?;
    if let Value::String(raw_json) = &payload
        && let Ok(parsed) = serde_json::from_str::<Value>(raw_json)
    {
        payload = parsed;
    }

    let transaction_value = extract_transaction_payload(&payload)
        .ok_or_else(|| SignerError::InvalidInput("Missing transaction object for Tron broadcast".to_string()))?;
    let transaction: TronTransaction = serde_json::from_value(transaction_value)?;
    let raw_data_hex = match &input.metadata {
        TransactionLoadMetadata::Tron {
            raw_data_hex: Some(raw_data_hex),
            ..
        } => raw_data_hex.clone(),
        _ => transaction
            .raw_data_hex
            .clone()
            .ok_or_else(|| SignerError::InvalidInput("Missing raw_data_hex in Tron transaction payload".to_string()))?,
    };

    let metadata = PayloadMetadata {
        payload,
        output_type: extra.output_type.clone(),
        output_action: extra.output_action.clone(),
        raw_data_hex,
    };
    Ok((transaction, metadata))
}

fn extract_transaction_payload(value: &Value) -> Option<Value> {
    match value {
        Value::Object(map) => {
            if map.get("raw_data_hex").is_some() {
                return Some(value.clone());
            }
            if let Some(transaction) = map.get("transaction")
                && let Some(found) = extract_transaction_payload(transaction)
            {
                return Some(found);
            }
            map.values().find_map(extract_transaction_payload)
        }
        Value::Array(values) => values.iter().find_map(extract_transaction_payload),
        _ => None,
    }
}

fn apply_signature(payload: Value, signature_hex: &str) -> Result<Value, SignerError> {
    let transaction_value = extract_transaction_payload(&payload)
        .ok_or_else(|| SignerError::InvalidInput("Missing transaction object for Tron broadcast".to_string()))?;
    let mut transaction: TronTransaction = serde_json::from_value(transaction_value)?;
    if !transaction.signature.is_empty() {
        return Err(SignerError::InvalidInput("Tron multisig not supported for WalletConnect signing".to_string()));
    }
    transaction.signature = vec![signature_hex.to_string()];
    let updated_transaction = serde_json::to_value(transaction)?;
    match payload {
        Value::Object(mut map) => {
            if map.get("raw_data_hex").is_some() {
                Ok(updated_transaction)
            } else if map.get("transaction").is_some() {
                map.insert("transaction".to_string(), updated_transaction);
                Ok(Value::Object(map))
            } else {
                Err(SignerError::InvalidInput("Missing raw_data_hex in Tron transaction payload".to_string()))
            }
        }
        _ => Err(SignerError::InvalidInput("Invalid Tron transaction payload".to_string())),
    }
}
