use gem_hash::message::hash_personal_message;

use crate::ETHEREUM_MESSAGE_PREFIX;

pub fn eip191_hash_message(message: &[u8]) -> [u8; 32] {
    hash_personal_message(ETHEREUM_MESSAGE_PREFIX, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eip191_hash_message() {
        let hash = eip191_hash_message(b"hello world");
        assert_eq!(hex::encode(hash), "d9eba16ed0ecae432b71fe008c98cc872bb4cc214d3220a36f365326cf807d68");
    }
}
