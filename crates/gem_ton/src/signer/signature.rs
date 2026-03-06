use std::time::{SystemTime, UNIX_EPOCH};

use primitives::SignerError;
use signer::Signer;

use super::types::{TonSignMessageData, TonSignResult};

pub fn sign_personal(data: &[u8], private_key: &[u8]) -> Result<TonSignResult, SignerError> {
    let ton_data = TonSignMessageData::from_bytes(data)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let digest = ton_data.build_sign_data_hash(timestamp)?;

    let (signature, public_key) = Signer::sign_ed25519_with_public_key(&digest, private_key).map_err(|e| SignerError::InvalidInput(e.to_string()))?;
    Ok(TonSignResult { signature, public_key, timestamp })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::TonSignDataPayload;

    #[test]
    fn test_sign_ton_personal() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string());
        let data = ton_data.to_bytes();

        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();

        let result = sign_personal(&data, &private_key).unwrap();

        assert_eq!(result.signature.len(), 64);
        assert_eq!(result.public_key.len(), 32);
        assert!(result.timestamp > 0);
    }

    #[test]
    fn test_sign_ton_personal_rejects_invalid_key() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string());
        let data = ton_data.to_bytes();

        let result = sign_personal(&data, &[0u8; 16]);
        assert!(result.is_err());
    }
}
