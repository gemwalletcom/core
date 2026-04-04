use super::models::NearTransfer;
use primitives::SignerError;

const ED25519_KEY_TYPE: u8 = 0;
const TRANSFER_ACTION: u8 = 3;

pub(crate) fn encode_transfer(transfer: &NearTransfer, public_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    if public_key.len() != 32 {
        return Err(SignerError::invalid_input("Near public key must be 32 bytes"));
    }

    let mut data = Vec::new();
    write_string(&mut data, &transfer.signer_id);
    data.push(ED25519_KEY_TYPE);
    data.extend_from_slice(public_key);
    data.extend_from_slice(&transfer.nonce.to_le_bytes());
    write_string(&mut data, &transfer.receiver_id);
    data.extend_from_slice(&transfer.block_hash);
    data.extend_from_slice(&1u32.to_le_bytes());
    data.push(TRANSFER_ACTION);
    data.extend_from_slice(&transfer.deposit);
    Ok(data)
}

fn write_string(data: &mut Vec<u8>, value: &str) {
    data.extend_from_slice(&(value.len() as u32).to_le_bytes());
    data.extend_from_slice(value.as_bytes());
}

pub(crate) fn parse_u128_le(value: &str) -> Result<[u8; 16], SignerError> {
    let parsed = value.parse::<u128>().map_err(|_| SignerError::invalid_input("invalid Near amount"))?;
    Ok(parsed.to_le_bytes())
}
