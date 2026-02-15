use gem_hash::message::hash_personal_message;

const TRON_MESSAGE_PREFIX: &str = "\x19TRON Signed Message:\n";

pub fn tron_hash_message(message: &[u8]) -> [u8; 32] {
    hash_personal_message(TRON_MESSAGE_PREFIX, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_hash_message() {
        let hash = tron_hash_message(b"This is a message to be signed for Tron");
        assert_eq!(hex::encode(hash), "aa8faa6427ddbbcbcdd441df0adec9ddebc1188e0d4cce7a43a3d4bf9496acac");
    }
}
