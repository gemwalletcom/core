use crate::models::signing::AlgorandTransaction;
use crate::signer::serialization::{encode_signed_transaction, encode_transaction};
use primitives::SignerError;
use signer::{SignatureScheme, Signer};

const TX_TAG: &[u8; 2] = b"TX";

pub(crate) fn sign_transaction(transaction: &AlgorandTransaction, private_key: &[u8]) -> Result<String, SignerError> {
    let encoded = encode_transaction(transaction)?;

    let mut preimage = Vec::with_capacity(TX_TAG.len() + encoded.len());
    preimage.extend_from_slice(TX_TAG);
    preimage.extend_from_slice(&encoded);

    let signature = Signer::sign_digest(SignatureScheme::Ed25519, preimage, private_key.to_vec())?;
    let signed = encode_signed_transaction(&encoded, &signature);
    Ok(hex::encode(signed))
}
