use k256::ecdsa::SigningKey as SecpSigningKey;
use primitives::SignerError;

pub const SIGNATURE_LENGTH: usize = 65;
pub const RECOVERY_ID_INDEX: usize = SIGNATURE_LENGTH - 1;
const ETHEREUM_RECOVERY_ID_OFFSET: u8 = 27;

/// Returns (signature_bytes, recovery_id) where recovery_id ∈ {0, 1}.
pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, u8), SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::signing_error("Invalid Secp256k1 private key"))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|_| SignerError::signing_error("Failed to sign Secp256k1 digest"))?;
    Ok((signature.to_bytes().to_vec(), u8::from(recovery_id)))
}

/// Returns [r(32), s(32), v(1)] where v ∈ {0, 1}.
pub(crate) fn sign_digest_append_recovery(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let (rs, v) = sign_digest(digest, private_key)?;
    Ok([rs, vec![v]].concat())
}

/// Returns [r(32), s(32), v(1)] where v ∈ {27, 28} (Ethereum/Tron).
pub(crate) fn sign_eth_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let (rs, v) = sign_digest(digest, private_key)?;
    Ok([rs, vec![v + ETHEREUM_RECOVERY_ID_OFFSET]].concat())
}

pub fn public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::invalid_input("Invalid Secp256k1 private key"))?;
    Ok(signing_key.verifying_key().to_sec1_bytes().to_vec())
}

/// Apply Ethereum recovery id offset (+27) to a 65-byte signature. Idempotent.
pub fn apply_eth_recovery_id(signature: &mut [u8]) {
    if signature.len() != 65 {
        return;
    }
    let v = &mut signature[64];
    if *v < ETHEREUM_RECOVERY_ID_OFFSET {
        *v += ETHEREUM_RECOVERY_ID_OFFSET;
    }
}

#[cfg(test)]
mod tests {
    use super::{ETHEREUM_RECOVERY_ID_OFFSET, SecpSigningKey, apply_eth_recovery_id, sign_digest, sign_eth_digest};
    use crate::testkit::TEST_PRIVATE_KEY;
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
    const DIGEST: [u8; 32] = [7u8; 32];

    #[test]
    fn sign_digest_returns_raw_recovery_id() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, v) = sign_digest(&DIGEST, &private_key).unwrap();
        let signing_key = SecpSigningKey::from_slice(&private_key).unwrap();

        assert_eq!(rs.len(), 64);
        assert!(matches!(v, 0 | 1), "raw recovery id must be 0 or 1, got {v}");

        let recovery_id = RecoveryId::from_byte(v).unwrap();
        let signature = Signature::try_from(rs.as_slice()).unwrap();
        let recovered = VerifyingKey::recover_from_prehash(&DIGEST, &signature, recovery_id).unwrap();
        assert_eq!(recovered.to_sec1_bytes().to_vec(), signing_key.verifying_key().to_sec1_bytes().to_vec());
    }

    #[test]
    fn sign_eth_digest_applies_offset() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let (rs, v) = sign_digest(&DIGEST, &private_key).unwrap();
        let signature = sign_eth_digest(&DIGEST, &private_key).unwrap();

        assert_eq!(rs, &signature[..64]);
        assert_eq!(v + ETHEREUM_RECOVERY_ID_OFFSET, signature[64]);
    }

    #[test]
    fn apply_recovery_id_offset() {
        let mut sig = vec![0u8; 65];

        sig[64] = 0;
        apply_eth_recovery_id(&mut sig);
        assert_eq!(sig[64], ETHEREUM_RECOVERY_ID_OFFSET);
        apply_eth_recovery_id(&mut sig);
        assert_eq!(sig[64], ETHEREUM_RECOVERY_ID_OFFSET);

        sig[64] = 1;
        apply_eth_recovery_id(&mut sig);
        assert_eq!(sig[64], 1 + ETHEREUM_RECOVERY_ID_OFFSET);
        apply_eth_recovery_id(&mut sig);
        assert_eq!(sig[64], 1 + ETHEREUM_RECOVERY_ID_OFFSET);
    }
}
