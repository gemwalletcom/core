use primitives::SignerError;
use signer::Ed25519KeyPair;

use super::types::{TonSignMessageData, TonSignResult};

pub fn sign_personal(data: &[u8], private_key: &[u8], timestamp: u64) -> Result<TonSignResult, SignerError> {
    let ton_data = TonSignMessageData::from_bytes(data)?;
    let digest = ton_data.hash(timestamp)?;

    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    Ok(TonSignResult {
        signature: key_pair.sign(&digest).to_vec(),
        public_key: key_pair.public_key_bytes.to_vec(),
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::TonSignDataPayload;
    use crate::signer::testkit::TEST_ADDRESS;

    #[test]
    fn test_sign_ton_personal() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), TEST_ADDRESS.to_string());
        let data = ton_data.to_bytes();

        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();
        let timestamp = 1234567890u64;

        let result = sign_personal(&data, &private_key, timestamp).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "3fe42db1d77534ba52d43240cf6b84b36eb1c53a28e3370c5872f37558cee9b758b9f93a8740c84ee4190b99de83901dcb9d5b42b1c7826b3836236ef5cd3a0f"
        );
        assert_eq!(hex::encode(&result.public_key), "d369452197c2a56481e5e2d3e8bf03de2349f67a63151956822208c2334adee2");
        assert_eq!(result.timestamp, timestamp);
    }

    #[test]
    fn test_sign_ton_personal_rejects_invalid_key() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), TEST_ADDRESS.to_string());
        let data = ton_data.to_bytes();

        let result = sign_personal(&data, &[0u8; 16], 1234567890);
        assert!(result.is_err());
    }
}
