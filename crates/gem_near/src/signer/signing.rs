use super::models::NearTransfer;
use super::serialization::encode_transfer;
use gem_encoding::encode_base64;
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::{ED25519_KEY_TYPE, Ed25519KeyPair};

pub fn sign_transfer(transfer: &NearTransfer, private_key: &[u8]) -> Result<String, SignerError> {
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let encoded = encode_transfer(transfer, &key_pair.public_key_bytes);
    let digest = sha256(&encoded);
    let signature = key_pair.sign(&digest);

    let mut signed = encoded;
    signed.push(ED25519_KEY_TYPE);
    signed.extend_from_slice(&signature);
    Ok(encode_base64(&signed))
}
