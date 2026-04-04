use super::models::NearTransfer;
use super::serialization::encode_transfer;
use base64::{Engine, engine::general_purpose::STANDARD};
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::Signer;

pub(crate) fn sign_transfer(transfer: &NearTransfer, private_key: &[u8]) -> Result<String, SignerError> {
    let public_key = Signer::ed25519_public_key(private_key)?;
    let preimage = encode_transfer(transfer, public_key.as_slice())?;
    let digest = sha256(&preimage);
    let (signature, _) = Signer::sign_ed25519_with_public_key(&digest, private_key)?;

    let mut signed_transaction = preimage;
    signed_transaction.push(0);
    signed_transaction.extend_from_slice(&signature);
    Ok(STANDARD.encode(signed_transaction))
}
