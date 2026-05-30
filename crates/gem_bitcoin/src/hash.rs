use bitcoin::hashes::{Hash, hash160 as bitcoin_hash160, sha256d};
use primitives::SignerError;

pub(crate) const HASH160_LEN: usize = 20;

pub(crate) fn double_sha256(bytes: &[u8]) -> [u8; 32] {
    sha256d::Hash::hash(bytes).to_byte_array()
}

pub(crate) fn hash160(bytes: &[u8]) -> [u8; HASH160_LEN] {
    bitcoin_hash160::Hash::hash(bytes).to_byte_array()
}

pub(crate) fn public_key_hash(public_key: &[u8]) -> [u8; HASH160_LEN] {
    hash160(public_key)
}

pub(crate) fn hash20(bytes: &[u8]) -> Result<[u8; HASH160_LEN], SignerError> {
    bytes.try_into().map_err(SignerError::from_display)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashes() {
        assert_eq!(hex::encode(double_sha256(b"")), "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456");
        assert_eq!(hex::encode(hash160(b"")), "b472a266d0bd89c13706a4132ccfb16f7c3b9fcb");
        assert_eq!(public_key_hash(b""), hash160(b""));
        assert_eq!(hash20(&[1u8; HASH160_LEN]).unwrap(), [1u8; HASH160_LEN]);
        assert!(hash20(&[1u8; HASH160_LEN - 1]).is_err());
    }
}
