use crate::error::SignerError;
use k256::ecdsa::SigningKey as SecpSigningKey;

pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::new("Invalid Secp256k1 private key"))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|_| SignerError::new("Failed to sign Secp256k1 digest"))?;

    let mut out = signature.to_bytes().to_vec();
    out.push(u8::from(recovery_id));
    Ok(out)
}
