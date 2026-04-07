use sha3::{Digest, Keccak256};

pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    Keccak256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        assert_eq!(
            hex::encode(keccak256(b"hello")),
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
        assert_eq!(
            hex::encode(keccak256(b"")),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }
}
