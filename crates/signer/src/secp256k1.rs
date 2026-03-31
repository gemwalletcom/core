use k256::ecdsa::SigningKey as SecpSigningKey;
use primitives::SignerError;

const ETHEREUM_RECOVERY_ID_OFFSET: u8 = 27;

pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::signing_error("Invalid Secp256k1 private key"))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|_| SignerError::signing_error("Failed to sign Secp256k1 digest"))?;

    let mut out = signature.to_bytes().to_vec();
    out.push(u8::from(recovery_id) + ETHEREUM_RECOVERY_ID_OFFSET);
    Ok(out)
}

pub fn public_key_from_private(private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| SignerError::invalid_input("Invalid Secp256k1 private key"))?;
    Ok(signing_key.verifying_key().to_sec1_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::{ETHEREUM_RECOVERY_ID_OFFSET, SecpSigningKey, sign_digest};
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    #[test]
    fn sign_digest_uses_ethereum_recovery_id_values() {
        let digest = [7u8; 32];
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();
        let signature = sign_digest(&digest, &private_key).unwrap();
        let signing_key = SecpSigningKey::from_slice(&private_key).unwrap();

        assert_eq!(signature.len(), 65);

        match signature[64] {
            27 | 28 => (),
            value => panic!("unexpected recovery id: {value}"),
        }

        let recovery_id = RecoveryId::from_byte(signature[64] - ETHEREUM_RECOVERY_ID_OFFSET).unwrap();
        let signature = Signature::try_from(&signature[..64]).unwrap();
        let recovered = VerifyingKey::recover_from_prehash(&digest, &signature, recovery_id).unwrap();

        assert_eq!(recovered.to_sec1_bytes().to_vec(), signing_key.verifying_key().to_sec1_bytes().to_vec());
    }
}
