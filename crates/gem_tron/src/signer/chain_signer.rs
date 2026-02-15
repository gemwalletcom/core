use super::transaction::{TronPayload, TronTransaction};
use gem_hash::sha2::sha256;
use primitives::{ChainSigner, SignerError, TransactionLoadInput, TransferDataOutputAction, TransferDataOutputType, hex::decode_hex};
use serde_json::Value;
use signer::{SignatureScheme, Signer};

struct PayloadMetadata {
    payload: Value,
    output_type: TransferDataOutputType,
    output_action: TransferDataOutputAction,
}

pub struct TronChainSigner;

impl ChainSigner for TronChainSigner {
    fn sign_data(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_data(input, private_key)
    }
}

fn sign_data(input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
    let (transaction, metadata) = get_transaction(input)?;
    let raw_data_hex = transaction
        .raw_data_hex
        .as_deref()
        .ok_or_else(|| SignerError::invalid_input("Missing raw_data_hex in Tron transaction payload"))?;
    let raw_bytes = decode_hex(raw_data_hex)?;
    let digest = sha256(&raw_bytes);
    let signature = Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec()).map_err(|e| SignerError::signing_error(e.to_string()))?;
    let signature_hex = hex::encode(signature);

    match metadata.output_type {
        TransferDataOutputType::Signature => Ok(signature_hex),
        TransferDataOutputType::EncodedTransaction => {
            let payload = apply_signature(metadata.payload, &signature_hex)?;
            let result_payload = match metadata.output_action {
                TransferDataOutputAction::Send => payload
                    .get("transaction")
                    .cloned()
                    .ok_or_else(|| SignerError::invalid_input("Missing transaction object for Tron broadcast"))?,
                TransferDataOutputAction::Sign => payload,
            };

            Ok(serde_json::to_string(&result_payload)?)
        }
    }
}

fn get_transaction(input: &TransactionLoadInput) -> Result<(TronTransaction, PayloadMetadata), SignerError> {
    let extra = input.get_data_extra().map_err(SignerError::invalid_input)?;
    let data = extra.data.as_ref().ok_or_else(|| SignerError::invalid_input("Missing transaction data"))?;

    let payload: TronPayload = serde_json::from_slice(data).map_err(|_| SignerError::invalid_input("Invalid Tron transaction payload"))?;
    let transaction = payload.transaction.clone();
    let payload_value = serde_json::to_value(&payload).map_err(|_| SignerError::invalid_input("Invalid Tron transaction payload"))?;
    let metadata = PayloadMetadata {
        payload: payload_value,
        output_type: extra.output_type.clone(),
        output_action: extra.output_action.clone(),
    };
    Ok((transaction, metadata))
}

fn apply_signature(payload: Value, signature_hex: &str) -> Result<Value, SignerError> {
    let mut payload: TronPayload = serde_json::from_value(payload).map_err(|_| SignerError::invalid_input("Invalid Tron transaction payload"))?;
    payload.transaction.signature = Some(vec![signature_hex.to_string()]);
    payload.signature = Some(signature_hex.to_string());
    Ok(serde_json::to_value(payload)?)
}
