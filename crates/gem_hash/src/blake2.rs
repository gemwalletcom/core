pub fn blake2b_224(bytes: &[u8]) -> [u8; 28] {
    blake2b(bytes)
}

pub fn blake2b_256(bytes: &[u8]) -> [u8; 32] {
    blake2b(bytes)
}

pub fn blake2b_256_personal(bytes: &[u8], personal: &[u8; 16]) -> [u8; 32] {
    let hash = blake2b_simd::Params::new().hash_length(32).personal(personal).hash(bytes);
    let mut output = [0u8; 32];
    output.copy_from_slice(hash.as_bytes());
    output
}

pub fn blake2b_512(bytes: &[u8]) -> [u8; 64] {
    blake2b(bytes)
}

fn blake2b<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let hash = blake2b_simd::Params::new().hash_length(N).hash(bytes);
    let mut output = [0u8; N];
    output.copy_from_slice(hash.as_bytes());
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2b_256_personal() {
        assert_eq!(
            hex::encode(blake2b_256_personal(&[], b"ZTxIdSaplingHash")),
            "6f2fc8f98feafd94e74a0df4bed74391ee0b5a69945e4ced8ca8a095206f00ae"
        );
    }
}
