use crate::models::signing::AlgorandTransaction;
use crate::signer::serialization::{encode_signed_transaction, encode_transaction};
use primitives::SignerError;
use signer::{SignatureScheme, Signer};

const ALGORAND_TRANSACTION_TAG: [u8; 2] = *b"TX";

pub(crate) fn sign_transaction(transaction: &AlgorandTransaction, private_key: &[u8]) -> Result<String, SignerError> {
    let encoded_transaction = encode_transaction(transaction)?;

    let mut preimage = Vec::with_capacity(ALGORAND_TRANSACTION_TAG.len() + encoded_transaction.len());
    preimage.extend_from_slice(&ALGORAND_TRANSACTION_TAG);
    preimage.extend_from_slice(&encoded_transaction);

    let signature = Signer::sign_digest(SignatureScheme::Ed25519, preimage, private_key.to_vec())?;
    let signed = encode_signed_transaction(&encoded_transaction, &signature)?;
    Ok(hex::encode(signed))
}
