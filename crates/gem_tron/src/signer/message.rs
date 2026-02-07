use gem_hash::keccak::keccak256;

const TRON_MESSAGE_PREFIX: &str = "\x19TRON Signed Message:\n";

pub fn tron_hash_message(message: &[u8]) -> [u8; 32] {
    let prefix = format!("{TRON_MESSAGE_PREFIX}{}", message.len());
    let mut data = Vec::with_capacity(prefix.len() + message.len());
    data.extend_from_slice(prefix.as_bytes());
    data.extend_from_slice(message);
    keccak256(&data)
}
