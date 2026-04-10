use super::models::NearTransfer;
use signer::ED25519_KEY_TYPE;

const TRANSFER_ACTION: u8 = 3;

pub fn encode_transfer(transfer: &NearTransfer, public_key: &[u8; 32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);
    write_string(&mut buf, &transfer.signer_id);
    buf.push(ED25519_KEY_TYPE);
    buf.extend_from_slice(public_key);
    buf.extend_from_slice(&transfer.nonce.to_le_bytes());
    write_string(&mut buf, &transfer.receiver_id);
    buf.extend_from_slice(&transfer.block_hash);
    // 1 action
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.push(TRANSFER_ACTION);
    buf.extend_from_slice(&transfer.deposit);
    buf
}

fn write_string(buf: &mut Vec<u8>, value: &str) {
    buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
    buf.extend_from_slice(value.as_bytes());
}
