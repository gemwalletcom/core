use crate::models::signing::StellarTransaction;
use crate::signer::serialization::encode_transaction;
use gem_encoding::encode_base64;
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::Ed25519KeyPair;

const NETWORK_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";
const ENVELOPE_TYPE_TX: [u8; 4] = 2u32.to_be_bytes();
const SIGNATURE_COUNT: [u8; 4] = 1u32.to_be_bytes();

pub(crate) fn sign_transaction(transaction: &StellarTransaction, private_key: &[u8]) -> Result<String, SignerError> {
    let encoded = encode_transaction(transaction);
    let network_id = sha256(NETWORK_PASSPHRASE.as_bytes());

    let mut preimage = Vec::with_capacity(network_id.len() + ENVELOPE_TYPE_TX.len() + encoded.len());
    preimage.extend_from_slice(&network_id);
    preimage.extend_from_slice(&ENVELOPE_TYPE_TX);
    preimage.extend_from_slice(&encoded);

    let digest = sha256(&preimage);
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let signature = key_pair.sign(&digest);

    let mut envelope = encoded;
    envelope.extend_from_slice(&SIGNATURE_COUNT);
    envelope.extend_from_slice(&key_pair.public_key_bytes[28..32]);
    envelope.extend_from_slice(&(signature.len() as u32).to_be_bytes());
    envelope.extend_from_slice(&signature);
    Ok(encode_base64(&envelope))
}
