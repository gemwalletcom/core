use gem_hash::keccak::keccak256;

pub fn namehash(name: &str) -> Vec<u8> {
    if name.is_empty() {
        return vec![0u8; 32];
    }
    let mut hash = vec![0u8; 32];
    for label in name.rsplit('.') {
        hash.append(&mut keccak256(label.as_bytes()).to_vec());
        hash = keccak256(hash.as_slice()).to_vec();
    }
    hash
}

#[cfg(test)]
mod test {
    use super::namehash;
    #[test]
    fn test_namehash() {
        // Test cases from https://github.com/ethereum/ercs/blob/master/ERCS/erc-137.md
        let cases = vec![
            ("", hex::decode("0000000000000000000000000000000000000000000000000000000000000000")),
            ("eth", hex::decode("93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae")),
            ("foo.eth", hex::decode("de9b09fd7c5f901e23a3f19fecc54828e9c848539801e86591bd9801b019f84f")),
        ];

        for (name, expected_namehash) in cases {
            let namehash: &[u8] = &namehash(name);
            assert_eq!(namehash, expected_namehash.unwrap());
        }
    }
}
