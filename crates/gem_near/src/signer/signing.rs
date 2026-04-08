use super::models::NearTransfer;
use super::serialization::{ED25519_KEY_TYPE, encode_transfer};
use base64::{Engine, engine::general_purpose::STANDARD};
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::Ed25519KeyPair;

pub fn sign_transfer(transfer: &NearTransfer, private_key: &[u8]) -> Result<String, SignerError> {
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let encoded = encode_transfer(transfer, &key_pair.public_key_bytes);
    let digest = sha256(&encoded);
    let signature = key_pair.sign(&digest);

    let mut signed = encoded;
    signed.push(ED25519_KEY_TYPE);
    signed.extend_from_slice(&signature);
    Ok(STANDARD.encode(&signed))
}
