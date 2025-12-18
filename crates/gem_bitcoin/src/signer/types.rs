use primitives::SignerError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const BITCOIN_MESSAGE_PREFIX: &[u8] = b"\x18Bitcoin Signed Message:\n";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BitcoinSignMessageData {
    pub message: String,
    pub address: String,
}

impl BitcoinSignMessageData {
    pub fn new(message: String, address: String) -> Self {
        Self { message, address }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, SignerError> {
        serde_json::from_slice(data).map_err(SignerError::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn hash(&self) -> Vec<u8> {
        let message = self.message.as_bytes();
        let varint = encode_varint(message.len());

        let mut data = Vec::with_capacity(BITCOIN_MESSAGE_PREFIX.len() + varint.len() + message.len());
        data.extend_from_slice(BITCOIN_MESSAGE_PREFIX);
        data.extend_from_slice(&varint);
        data.extend_from_slice(message);

        let first_hash = Sha256::digest(&data);
        Sha256::digest(first_hash).to_vec()
    }
}

fn encode_varint(n: usize) -> Vec<u8> {
    if n < 0xfd {
        vec![n as u8]
    } else if n <= 0xffff {
        let b = (n as u16).to_le_bytes();
        vec![0xfd, b[0], b[1]]
    } else if n <= 0xffffffff {
        let b = (n as u32).to_le_bytes();
        vec![0xfe, b[0], b[1], b[2], b[3]]
    } else {
        let b = (n as u64).to_le_bytes();
        vec![0xff, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
    }
}

#[derive(Serialize)]
pub struct BitcoinSignDataResponse {
    address: String,
    signature: String,
}

impl BitcoinSignDataResponse {
    pub fn new(address: String, signature: String) -> Self {
        Self { address, signature }
    }

    pub fn to_json(&self) -> Result<String, SignerError> {
        serde_json::to_string(self).map_err(SignerError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_to_bytes() {
        let data = BitcoinSignMessageData::new("Hello Bitcoin".to_string(), "bc1qtest".to_string());

        let bytes = data.to_bytes();
        let parsed = BitcoinSignMessageData::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.message, "Hello Bitcoin");
        assert_eq!(parsed.address, "bc1qtest");
    }

    #[test]
    fn test_hash() {
        let data = BitcoinSignMessageData::new("Hello Bitcoin".to_string(), "bc1qtest".to_string());
        let hash = data.hash();

        assert_eq!(hash.len(), 32);
        assert_eq!(
            hex::encode(&hash),
            "93a4e556613458adb2019c52d7dbaff7a7261da4bc4b8b3f8b9c5f098209de37"
        );
    }

    #[test]
    fn test_response_to_json() {
        let response = BitcoinSignDataResponse::new("bc1qtest".to_string(), "27abcdef".to_string());

        let json = response.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["address"], "bc1qtest");
        assert_eq!(parsed["signature"], "27abcdef");
    }
}
