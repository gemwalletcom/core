use crate::models::{Ed25519Authenticator, RawTransaction, SignedTransaction, SubmitTransactionBcsRequest, TransactionAuthenticator, TransactionPayloadBCS};
use gem_hash::sha3::sha3_256;
use hex::encode;
use primitives::SignerError;
use signer::Signer;
use std::time::SystemTime;

use super::{AccountAddress, EntryFunction};

const RAW_TRANSACTION_SALT: &[u8] = b"APTOS::RawTransaction";
const MESSAGE_SALT: &[u8] = b"APTOS::Message";

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
    let raw_tx_bytes = bcs::to_bytes(raw_tx).map_err(|err| SignerError::InvalidInput(format!("Failed to encode Aptos transaction: {err}")))?;
    let seed = sha3_256(RAW_TRANSACTION_SALT);
    let mut preimage = Vec::with_capacity(seed.len() + raw_tx_bytes.len());
    preimage.extend_from_slice(&seed);
    preimage.extend_from_slice(&raw_tx_bytes);

    Signer::sign_ed25519_with_public_key(&preimage, private_key).map_err(|err| SignerError::InvalidInput(err.to_string()))
}

pub fn sign_message(message: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SignerError> {
    let seed = sha3_256(MESSAGE_SALT);
    let mut preimage = Vec::with_capacity(seed.len() + message.len());
    preimage.extend_from_slice(&seed);
    preimage.extend_from_slice(message);

    Signer::sign_ed25519_with_public_key(&preimage, private_key).map_err(|err| SignerError::InvalidInput(err.to_string()))
}

pub fn build_submit_transaction_bcs(raw_tx: RawTransaction, signature: Vec<u8>, public_key: Vec<u8>) -> Result<String, SignerError> {
    let signed = SignedTransaction {
        raw_tx,
        authenticator: TransactionAuthenticator::Ed25519(Ed25519Authenticator {
            public_key: ensure_length(public_key, 32, "public key")?,
            signature: ensure_length(signature, 64, "signature")?,
        }),
    };
    let bcs_bytes = bcs::to_bytes(&signed).map_err(|err| SignerError::InvalidInput(format!("Failed to encode Aptos signed transaction: {err}")))?;
    let request = SubmitTransactionBcsRequest {
        bcs: encode(bcs_bytes),
        bcs_encoding: "hex".to_string(),
    };

    serde_json::to_string(&request).map_err(|err| SignerError::InvalidInput(err.to_string()))
}

pub fn expiration_timestamp_secs() -> Result<u64, SignerError> {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| SignerError::InvalidInput("Invalid system time".to_string()))?;
    Ok(now.as_secs() + 3_600)
}

fn ensure_length(input: Vec<u8>, expected: usize, label: &str) -> Result<Vec<u8>, SignerError> {
    if input.len() != expected {
        return Err(SignerError::InvalidInput(format!(
            "Invalid Aptos {label} length: expected {expected}, got {}",
            input.len()
        )));
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::EntryFunctionPayload;
    use ed25519_dalek::{Signature, SigningKey, Verifier};
    use serde_json::Value;

    fn sample_raw_tx() -> RawTransaction {
        let payload = EntryFunctionPayload {
            payload_type: "entry_function_payload".to_string(),
            function: "0x1::aptos_account::transfer".to_string(),
            type_arguments: Vec::new(),
            arguments: vec![
                Value::String("0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef".to_string()),
                Value::String("100".to_string()),
            ],
        };
        let entry_function = payload.to_entry_function(Some(&["address", "u64"])).expect("entry function");
        build_raw_transaction(
            AccountAddress::from_hex("0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef").unwrap(),
            1,
            entry_function,
            1500,
            100,
            1700000000,
            1,
        )
    }

    #[test]
    fn sign_raw_transaction_verifies_against_signing_message() {
        let raw_tx = sample_raw_tx();
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();
        let (signature, public_key) = sign_raw_transaction(&raw_tx, &private_key).expect("signature");

        let raw_tx_bytes = bcs::to_bytes(&raw_tx).expect("bcs");
        let seed = sha3_256(RAW_TRANSACTION_SALT);
        let mut preimage = Vec::with_capacity(seed.len() + raw_tx_bytes.len());
        preimage.extend_from_slice(&seed);
        preimage.extend_from_slice(&raw_tx_bytes);

        let signing_key = SigningKey::from_bytes(&private_key.try_into().unwrap());
        assert_eq!(public_key, signing_key.verifying_key().to_bytes().to_vec());
        let signature = Signature::from_bytes(&signature.try_into().unwrap());
        signing_key.verifying_key().verify(&preimage, &signature).expect("signature should verify");
    }

    #[test]
    fn build_submit_transaction_bcs_roundtrip() {
        let raw_tx = sample_raw_tx();
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();
        let (signature, public_key) = sign_raw_transaction(&raw_tx, &private_key).expect("signature");

        let json = build_submit_transaction_bcs(raw_tx.clone(), signature.clone(), public_key.clone()).expect("bcs");
        let request: SubmitTransactionBcsRequest = serde_json::from_str(&json).expect("json");
        assert_eq!(request.bcs_encoding, "hex");

        let bytes = hex::decode(request.bcs).expect("hex");
        let signed: SignedTransaction = bcs::from_bytes(&bytes).expect("bcs decode");
        assert_eq!(signed.raw_tx, raw_tx);

        match signed.authenticator {
            TransactionAuthenticator::Ed25519(authenticator) => {
                assert_eq!(authenticator.public_key, public_key);
                assert_eq!(authenticator.signature, signature);
            }
        }
    }
}
