use curve25519_dalek::{constants::ED25519_BASEPOINT_TABLE, scalar::Scalar};
use primitives::SignerError;
use sha2::{Digest, Sha512};
use zeroize::Zeroize;

const EXTENDED_PRIVATE_KEY_LENGTH: usize = 192;
const KEY_LENGTH: usize = 32;
const EXTENSION_OFFSET: usize = 32;

pub(super) struct CardanoExtendedKeyPair {
    secret: [u8; KEY_LENGTH],
    extension: [u8; KEY_LENGTH],
}

impl Drop for CardanoExtendedKeyPair {
    fn drop(&mut self) {
        self.secret.zeroize();
        self.extension.zeroize();
    }
}

impl CardanoExtendedKeyPair {
    pub(super) fn from_private_key(private_key: &[u8]) -> Result<Self, SignerError> {
        if private_key.len() != EXTENDED_PRIVATE_KEY_LENGTH {
            return SignerError::invalid_input_err("invalid Cardano private key length");
        }

        Ok(Self {
            secret: read_key(private_key, 0),
            extension: read_key(private_key, EXTENSION_OFFSET),
        })
    }

    pub(super) fn public_key(&self) -> [u8; KEY_LENGTH] {
        public_key_from_secret(self.secret)
    }

    pub(super) fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signing_scalar = Scalar::from_bytes_mod_order(self.secret);
        let public_key = self.public_key();
        let r = scalar_from_hash(&[self.extension.as_slice(), message]);
        let r_bytes = (&r * ED25519_BASEPOINT_TABLE).compress().to_bytes();
        let k = scalar_from_hash(&[r_bytes.as_slice(), public_key.as_slice(), message]);
        let s = k * signing_scalar + r;

        let mut signature = [0u8; 64];
        signature[0..KEY_LENGTH].copy_from_slice(&r_bytes);
        signature[KEY_LENGTH..].copy_from_slice(&s.to_bytes());
        signature
    }
}

fn read_key(bytes: &[u8], offset: usize) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    key.copy_from_slice(&bytes[offset..offset + KEY_LENGTH]);
    key
}

fn public_key_from_secret(secret: [u8; KEY_LENGTH]) -> [u8; KEY_LENGTH] {
    let scalar = Scalar::from_bytes_mod_order(secret);
    (&scalar * ED25519_BASEPOINT_TABLE).compress().to_bytes()
}

fn scalar_from_hash(parts: &[&[u8]]) -> Scalar {
    let mut hasher = Sha512::new();
    for part in parts {
        hasher.update(part);
    }
    let bytes: [u8; 64] = hasher.finalize().into();
    Scalar::from_bytes_mod_order_wide(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardano_extended_key_pair() {
        let private_key = hex::decode(
            "d809b1b4b4c74734037f76aace501730a3fe2fca30b5102df99ad3f7c0103e48\
             d54cde47e9041b31f3e6873d700d83f7a937bea746dadfa2c5b0a6a92502356c\
             69272d81c376382b8a87c21370a7ae9618df8da708d1a9490939ec54ebe43000\
             1111111111111111111111111111111111111111111111111111111111111111\
             1111111111111111111111111111111111111111111111111111111111111111\
             1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();

        let key_pair = CardanoExtendedKeyPair::from_private_key(&private_key).unwrap();

        assert_eq!(hex::encode(key_pair.public_key()), "e6f04522f875c1563682ca876ddb04c2e2e3ae718e3ff9f11c03dd9f9dccf698");
        assert_eq!(
            hex::encode(key_pair.sign(b"Hello world")),
            "1096ddcfb2ad21a4c0d861ef3fabe18841e8de88105b0d8e36430d7992c588634ead4100c32b2800b31b65e014d54a8238bdda63118d829bf0bcf1b631e86f0e"
        );
    }

    #[test]
    fn test_cardano_extended_key_pair_rejects_invalid_length() {
        assert!(CardanoExtendedKeyPair::from_private_key(&[0u8; 32]).is_err());
    }
}
