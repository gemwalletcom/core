use super::models::NearTransfer;
use super::serialization::{NEAR_ED25519_KEY_TYPE, encode_transfer};
use base64::{Engine, engine::general_purpose::STANDARD};
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::Ed25519KeyPair;

pub(crate) fn sign_transfer(transfer: &NearTransfer, private_key: &[u8]) -> Result<String, SignerError> {
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let preimage = encode_transfer(transfer, &key_pair.public_key_bytes)?;
    let digest = sha256(&preimage);
    let signature = key_pair.sign(&digest);

    let mut signed_transaction = preimage;
    signed_transaction.push(NEAR_ED25519_KEY_TYPE);
    signed_transaction.extend_from_slice(&signature);
    Ok(STANDARD.encode(signed_transaction))
}
