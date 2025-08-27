use alloy_primitives::hex;
use gem_hash::keccak::keccak256;
use rmp_serde;
use serde_json::Value;

pub fn action_hash(action: &Value, vault_address: Option<&str>, nonce: u64, expires_after: Option<u64>) -> Result<String, String> {
    // Serialize action with msgpack
    let mut data = rmp_serde::to_vec(action).map_err(|e| format!("Failed to serialize action: {e}"))?;

    // Add nonce (8 bytes, big endian)
    data.extend_from_slice(&nonce.to_be_bytes());

    // Handle vault address
    if let Some(vault) = vault_address {
        data.push(0x01);
        // Parse vault address and add as bytes
        let vault_bytes = hex::decode(vault.trim_start_matches("0x")).map_err(|e| format!("Invalid vault address: {e}"))?;
        data.extend_from_slice(&vault_bytes);
    } else {
        data.push(0x00);
    }

    // Handle expiration
    if let Some(expires) = expires_after {
        data.push(0x00);
        data.extend_from_slice(&expires.to_be_bytes());
    }

    // Calculate keccak256 hash
    let hash = keccak256(&data);
    Ok(hex::encode(hash))
}
