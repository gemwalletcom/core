use primitives::SignerError;
use signer::{SignatureScheme, Signer};

use super::types::{BitcoinSignDataResponse, BitcoinSignMessageData};

const SIGNATURE_LENGTH: usize = 65;
const RECOVERY_ID_INDEX: usize = SIGNATURE_LENGTH - 1;
const BIP137_P2WPKH_BASE: u8 = 39;

pub fn sign_personal(data: &[u8], private_key: &[u8]) -> Result<BitcoinSignDataResponse, SignerError> {
    let message = BitcoinSignMessageData::from_bytes(data)?;
    let hash = message.hash();

    let signed = Signer::sign_digest(SignatureScheme::Secp256k1, hash, private_key.to_vec())
        .map_err(|e| SignerError::InvalidInput(e.to_string()))?;

    // BIP137: [header(1), r(32), s(32)] from [r(32), s(32), recovery_id(1)]
    let recovery_id = signed[RECOVERY_ID_INDEX];
    let header = BIP137_P2WPKH_BASE + recovery_id;

    let mut signature = Vec::with_capacity(SIGNATURE_LENGTH);
    signature.push(header);
    signature.extend_from_slice(&signed[..RECOVERY_ID_INDEX]);

    Ok(BitcoinSignDataResponse::new(message.address, hex::encode(&signature)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_bitcoin_personal() {
        let data = BitcoinSignMessageData::new("Hello Bitcoin".to_string(), "bc1qtest".to_string()).to_bytes();
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sign_personal(&data, &private_key).unwrap().to_json().unwrap()).unwrap();
        assert_eq!(parsed["address"], "bc1qtest");
        assert!(!parsed["signature"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_sign_bitcoin_personal_rejects_invalid_key() {
        let data = BitcoinSignMessageData::new("Hello Bitcoin".to_string(), "bc1qtest".to_string()).to_bytes();
        assert!(sign_personal(&data, &[0u8; 16]).is_err());
    }
}
