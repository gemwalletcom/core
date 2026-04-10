use sha2::{Digest, Sha256, Sha512_256};

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn sha512_256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha512_256::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}
