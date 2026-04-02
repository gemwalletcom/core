use k256::ecdsa::SigningKey as SecpSigningKey;
use primitives::SignerError;

fn sign_prehash(digest: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, u8), SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::signing_error("Invalid Secp256k1 private key"))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|_| SignerError::signing_error("Failed to sign Secp256k1 digest"))?;
    Ok((signature.to_bytes().to_vec(), u8::from(recovery_id)))
}

/// Sign a digest and return [r(32), s(32), v(1)] where v ∈ {0, 1}.
pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let (rs, v) = sign_prehash(digest, private_key)?;
    Ok([rs, vec![v]].concat())
}

/// Sign a digest and return [r(32), s(32), v(1)] where v ∈ {27, 28} (Ethereum/Tron convention).
pub(crate) fn sign_ethereum_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let (rs, v) = sign_prehash(digest, private_key)?;
    Ok([rs, vec![v + 27]].concat())
}

pub fn public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::invalid_input("Invalid Secp256k1 private key"))?;
    Ok(signing_key.verifying_key().to_sec1_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::{SecpSigningKey, sign_digest, sign_ethereum_digest};
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    const PRIVATE_KEY: &str = "1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34";
    const DIGEST: [u8; 32] = [7u8; 32];

    #[test]
    fn sign_digest_returns_raw_recovery_id() {
        let private_key = hex::decode(PRIVATE_KEY).unwrap();
        let signature = sign_digest(&DIGEST, &private_key).unwrap();
        let signing_key = SecpSigningKey::from_slice(&private_key).unwrap();

        assert_eq!(signature.len(), 65);
        assert!(matches!(signature[64], 0 | 1), "raw recovery id must be 0 or 1, got {}", signature[64]);

        let recovery_id = RecoveryId::from_byte(signature[64]).unwrap();
        let sig = Signature::try_from(&signature[..64]).unwrap();
        let recovered = VerifyingKey::recover_from_prehash(&DIGEST, &sig, recovery_id).unwrap();
        assert_eq!(recovered.to_sec1_bytes().to_vec(), signing_key.verifying_key().to_sec1_bytes().to_vec());
    }

    #[test]
    fn sign_ethereum_digest_returns_ethereum_recovery_id() {
        let private_key = hex::decode(PRIVATE_KEY).unwrap();
        let signature = sign_ethereum_digest(&DIGEST, &private_key).unwrap();
        let signing_key = SecpSigningKey::from_slice(&private_key).unwrap();

        assert_eq!(signature.len(), 65);
        assert!(matches!(signature[64], 27 | 28), "ethereum recovery id must be 27 or 28, got {}", signature[64]);

        let recovery_id = RecoveryId::from_byte(signature[64] - 27).unwrap();
        let sig = Signature::try_from(&signature[..64]).unwrap();
        let recovered = VerifyingKey::recover_from_prehash(&DIGEST, &sig, recovery_id).unwrap();
        assert_eq!(recovered.to_sec1_bytes().to_vec(), signing_key.verifying_key().to_sec1_bytes().to_vec());
    }

    #[test]
    fn sign_digest_and_sign_ethereum_digest_rs_bytes_match() {
        let private_key = hex::decode(PRIVATE_KEY).unwrap();
        let raw = sign_digest(&DIGEST, &private_key).unwrap();
        let eth = sign_ethereum_digest(&DIGEST, &private_key).unwrap();

        // r and s must be identical; only v differs by exactly 27
        assert_eq!(&raw[..64], &eth[..64]);
        assert_eq!(raw[64] + 27, eth[64]);
    }
}
