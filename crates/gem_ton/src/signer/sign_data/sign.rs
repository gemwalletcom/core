use primitives::{Address as AddressTrait, SignerError};

use super::message::TonSignMessageData;
use crate::{
    address::Address,
    signer::signer::{TonSignResult, TonSigner},
};

impl TonSigner {
    pub fn sign_personal(&self, data: &[u8], timestamp: u64) -> Result<TonSignResult, SignerError> {
        let message_data = TonSignMessageData::from_bytes(data)?;
        Address::ensure_matches(Some(message_data.address.as_str()), &self.address().encode())?;
        let digest = message_data.hash_with_address(timestamp, self.address())?;

        Ok(TonSignResult {
            signature: self.sign(&digest).to_vec(),
            public_key: self.public_key().to_vec(),
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        address::base64_to_hex_address,
        signer::{
            TonSigner,
            sign_data::{TonSignDataPayload, TonSignMessageData},
            testkit::{TEST_PUBLIC_KEY, mock_cell, mock_signer, mock_signer_address},
        },
    };

    #[test]
    fn test_sign_ton_personal() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let message_data = TonSignMessageData::new(payload, "example.com".to_string(), mock_signer_address());
        let data = message_data.to_bytes();

        let result = mock_signer().sign_personal(&data, 1234567890).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "626168d23a7db9b8fa2716a7d3e3deeb3999f43dc6dfdd747206b6dba01058a4d785130710e2d4140730a643e2d633e76366f52dda8afd5c2acf4a6acb08ba0b"
        );
        assert_eq!(hex::encode(&result.public_key), TEST_PUBLIC_KEY);
        assert_eq!(result.timestamp, 1234567890);
    }

    #[test]
    fn test_sign_ton_personal_accepts_raw_address() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let address = base64_to_hex_address(&mock_signer_address()).unwrap();
        let message_data = TonSignMessageData::new(payload, "example.com".to_string(), address);
        let data = message_data.to_bytes();

        let result = mock_signer().sign_personal(&data, 1234567890).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "626168d23a7db9b8fa2716a7d3e3deeb3999f43dc6dfdd747206b6dba01058a4d785130710e2d4140730a643e2d633e76366f52dda8afd5c2acf4a6acb08ba0b"
        );
    }

    #[test]
    fn test_sign_ton_personal_rejects_invalid_key() {
        assert!(TonSigner::new(&[0u8; 16]).is_err());
    }

    #[test]
    fn test_sign_ton_personal_cell() {
        let payload = TonSignDataPayload::Cell {
            schema: "comment#00000000 text:SnakeData = InMsgBody;".to_string(),
            cell: mock_cell(),
        };
        let message_data = TonSignMessageData::new(payload, "example.com".to_string(), mock_signer_address());
        let data = message_data.to_bytes();

        let result = mock_signer().sign_personal(&data, 1234567890).unwrap();

        assert_eq!(
            hex::encode(&result.signature),
            "8ff07fcdb495d18b9274b8c837738f0217b56049c30ca622a075ca2ad5154b0ae9d364df087d368f78e25d15286a685816f325458f3127f27ca6f6880dac3903"
        );
        assert_eq!(result.timestamp, 1234567890);
    }

    #[test]
    fn test_sign_ton_personal_rejects_mismatched_address() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let message_data = TonSignMessageData::new(
            payload,
            "example.com".to_string(),
            "0:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        );
        let data = message_data.to_bytes();

        let result = mock_signer().sign_personal(&data, 1234567890);

        assert_eq!(result.err().unwrap().to_string(), "Invalid input: TON from does not match signer address");
    }
}
