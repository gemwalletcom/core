use blake2::{
    Blake2b, Blake2b512, Digest,
    digest::consts::{U28, U32},
};

type Blake2b224 = Blake2b<U28>;
type Blake2b256 = Blake2b<U32>;

pub fn blake2b_224(bytes: &[u8]) -> [u8; 28] {
    let mut hasher = Blake2b224::new();
    Digest::update(&mut hasher, bytes);
    hasher.finalize().into()
}

pub fn blake2b_256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2b256::new();
    Digest::update(&mut hasher, bytes);
    hasher.finalize().into()
}

pub fn blake2b_512(bytes: &[u8]) -> [u8; 64] {
    let mut hasher = Blake2b512::new();
    Digest::update(&mut hasher, bytes);
    hasher.finalize().into()
}
