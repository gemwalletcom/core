use crate::keccak::keccak256;

pub fn hash_personal_message(prefix: &str, message: &[u8]) -> [u8; 32] {
    let header = format!("{prefix}{}", message.len());
    keccak256(&[header.as_bytes(), message].concat())
}
