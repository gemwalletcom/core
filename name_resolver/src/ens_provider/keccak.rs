pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    hasher.update(bytes);

    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    hash
}
