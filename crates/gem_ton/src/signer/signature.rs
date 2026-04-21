use primitives::{Address as AddressTrait, SignerError};
use signer::Ed25519KeyPair;

use super::transaction::WalletV4R2;
use super::types::{TonSignMessageData, TonSignResult};
use crate::address::Address;

pub fn sign_personal(data: &[u8], private_key: &[u8], timestamp: u64) -> Result<TonSignResult, SignerError> {
    let ton_data = TonSignMessageData::from_bytes(data)?;
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let wallet = WalletV4R2::new(key_pair.public_key_bytes)?;
    Address::ensure_matches(ton_data.address.as_deref(), &wallet.address().encode())?;
    let digest = ton_data.hash_with_address(timestamp, wallet.address())?;

    Ok(TonSignResult {
        signature: key_pair.sign(&digest).to_vec(),
        public_key: key_pair.public_key_bytes.to_vec(),
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::base64_to_hex_address;
    use crate::signer::{BagOfCells, CellBuilder, TonSignDataPayload};

    const TEST_PRIVATE_KEY: &str = "1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34";
    const TEST_PUBLIC_KEY: &str = "d369452197c2a56481e5e2d3e8bf03de2349f67a63151956822208c2334adee2";

    fn signer_address() -> String {
        let public_key = <[u8; 32]>::try_from(hex::decode(TEST_PUBLIC_KEY).unwrap()).unwrap();
        WalletV4R2::new(public_key).unwrap().address().encode()
    }

    fn sample_cell() -> String {
        let mut builder = CellBuilder::new();
        builder.store_u32(32, 0).unwrap();
        BagOfCells::from_root(builder.build().unwrap()).to_base64(true).unwrap()
    }

    #[test]
    fn test_sign_ton_personal() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), Some(signer_address()));
        let data = ton_data.to_bytes();

        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let timestamp = 1234567890u64;

        let result = sign_personal(&data, &private_key, timestamp).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "626168d23a7db9b8fa2716a7d3e3deeb3999f43dc6dfdd747206b6dba01058a4d785130710e2d4140730a643e2d633e76366f52dda8afd5c2acf4a6acb08ba0b"
        );
        assert_eq!(hex::encode(&result.public_key), TEST_PUBLIC_KEY);
        assert_eq!(result.timestamp, timestamp);
    }

    #[test]
    fn test_sign_ton_personal_accepts_raw_address() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let address = base64_to_hex_address(&signer_address()).unwrap();
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), Some(address));
        let data = ton_data.to_bytes();

        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let result = sign_personal(&data, &private_key, 1234567890).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "626168d23a7db9b8fa2716a7d3e3deeb3999f43dc6dfdd747206b6dba01058a4d785130710e2d4140730a643e2d633e76366f52dda8afd5c2acf4a6acb08ba0b"
        );
    }

    #[test]
    fn test_sign_ton_personal_rejects_invalid_key() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), Some(signer_address()));
        let data = ton_data.to_bytes();

        let result = sign_personal(&data, &[0u8; 16], 1234567890);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_ton_personal_without_address() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), None);
        let data = ton_data.to_bytes();

        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let timestamp = 1234567890u64;

        let result = sign_personal(&data, &private_key, timestamp).unwrap();

        assert_eq!(hex::encode(&result.public_key), TEST_PUBLIC_KEY);
        assert_eq!(result.signature.len(), 64);
        assert_eq!(result.timestamp, timestamp);
    }

    #[test]
    fn test_sign_ton_personal_cell() {
        let payload = TonSignDataPayload::Cell {
            schema: "comment#00000000 text:SnakeData = InMsgBody;".to_string(),
            cell: sample_cell(),
        };
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string(), Some(signer_address()));
        let data = ton_data.to_bytes();

        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let result = sign_personal(&data, &private_key, 1234567890).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "8ff07fcdb495d18b9274b8c837738f0217b56049c30ca622a075ca2ad5154b0ae9d364df087d368f78e25d15286a685816f325458f3127f27ca6f6880dac3903"
        );
        assert_eq!(result.timestamp, 1234567890);
    }

    #[test]
    fn test_sign_ton_personal_rejects_mismatched_address() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let ton_data = TonSignMessageData::new(
            payload,
            "example.com".to_string(),
            Some("0:0000000000000000000000000000000000000000000000000000000000000000".to_string()),
        );
        let data = ton_data.to_bytes();

        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let result = sign_personal(&data, &private_key, 1234567890);

        assert_eq!(result.err().unwrap().to_string(), "Invalid input: TON from does not match signer address");
    }
}
