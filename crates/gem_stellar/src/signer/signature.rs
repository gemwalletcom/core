use crate::models::signing::StellarTransaction;
use crate::signer::serialization::encode_transaction;
use base64::{Engine, engine::general_purpose::STANDARD};
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::Signer;

const STELLAR_NETWORK_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";
// Stellar signs the transaction hash preimage as: network id + ENVELOPE_TYPE_TX + transaction XDR.
const STELLAR_ENVELOPE_TYPE_TX: [u8; 4] = 2u32.to_be_bytes();

pub(crate) fn sign_transaction(transaction: &StellarTransaction, private_key: &[u8]) -> Result<String, SignerError> {
    let encoded = encode_transaction(transaction);
    let preimage = signature_preimage(&encoded);
    let digest = sha256(&preimage);
    let (signature, _) = Signer::sign_ed25519_with_public_key(&digest, private_key)?;

    let mut envelope = encoded;
    envelope.extend_from_slice(&1u32.to_be_bytes());
    envelope.extend_from_slice(transaction.account.hint());
    envelope.extend_from_slice(&(signature.len() as u32).to_be_bytes());
    envelope.extend_from_slice(&signature);
    Ok(STANDARD.encode(envelope))
}

fn signature_preimage(encoded: &[u8]) -> Vec<u8> {
    let mut preimage = Vec::with_capacity(sha256(STELLAR_NETWORK_PASSPHRASE.as_bytes()).len() + 4 + encoded.len());
    preimage.extend_from_slice(&sha256(STELLAR_NETWORK_PASSPHRASE.as_bytes()));
    preimage.extend_from_slice(&STELLAR_ENVELOPE_TYPE_TX);
    preimage.extend_from_slice(encoded);
    preimage
}
