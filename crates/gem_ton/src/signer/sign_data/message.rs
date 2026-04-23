use crc::Crc;
use gem_hash::sha2::sha256;
use primitives::SignerError;
use serde::{Deserialize, Serialize};

use super::payload::TonSignDataPayload;
use crate::address::Address;
use crate::signer::cells::{BagOfCells, CellBuilder};

const SIGN_DATA_PREFIX: &[u8] = b"\xff\xffton-connect/sign-data/";
const CELL_SIGN_PREFIX: u32 = 0x75569022;
const SCHEMA_CRC32: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonSignMessageData {
    pub payload: TonSignDataPayload,
    pub domain: String,
    pub address: String,
}

impl TonSignMessageData {
    pub fn new(payload: TonSignDataPayload, domain: String, address: String) -> Self {
        Self { payload, domain, address }
    }

    pub fn from_value(payload: serde_json::Value, domain: String, address: String) -> Result<Self, SignerError> {
        let payload: TonSignDataPayload = serde_json::from_value(payload).map_err(SignerError::from)?;
        Ok(Self { payload, domain, address })
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, SignerError> {
        serde_json::from_slice(data).map_err(SignerError::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn hash(&self, timestamp: u64) -> Result<Vec<u8>, SignerError> {
        self.hash_with_address(timestamp, &Address::parse(&self.address)?)
    }

    pub fn hash_with_address(&self, timestamp: u64, address: &Address) -> Result<Vec<u8>, SignerError> {
        match &self.payload {
            TonSignDataPayload::Cell { schema, cell } => self.cell_payload_hash(schema, cell, address, timestamp),
            TonSignDataPayload::Text { .. } | TonSignDataPayload::Binary { .. } => {
                let domain_bytes = self.domain.as_bytes();
                let (type_prefix, payload_bytes) = self.payload.encode()?;

                let mut msg = Vec::new();
                msg.extend_from_slice(SIGN_DATA_PREFIX);
                msg.extend_from_slice(&address.workchain().to_be_bytes());
                msg.extend_from_slice(address.hash_part());
                msg.extend_from_slice(&(domain_bytes.len() as u32).to_be_bytes());
                msg.extend_from_slice(domain_bytes);
                msg.extend_from_slice(&timestamp.to_be_bytes());
                msg.extend_from_slice(type_prefix.as_bytes());
                msg.extend_from_slice(&(payload_bytes.len() as u32).to_be_bytes());
                msg.extend_from_slice(&payload_bytes);

                Ok(sha256(&msg).to_vec())
            }
        }
    }

    fn cell_payload_hash(&self, schema: &str, cell: &str, address: &Address, timestamp: u64) -> Result<Vec<u8>, SignerError> {
        let payload = BagOfCells::parse_base64_root(cell)?;
        let domain = self.dns_wire_domain()?;

        let mut domain_builder = CellBuilder::new();
        domain_builder.store_slice_snake(&domain)?;

        let mut builder = CellBuilder::new();
        builder
            .store_u32(32, CELL_SIGN_PREFIX)?
            .store_u32(32, SCHEMA_CRC32.checksum(schema.as_bytes()))?
            .store_u64(64, timestamp)?
            .store_address(address)?
            .store_child(domain_builder.build()?)?
            .store_reference(&payload)?;

        Ok(builder.build()?.hash.to_vec())
    }

    fn dns_wire_domain(&self) -> Result<Vec<u8>, SignerError> {
        let mut encoded = Vec::with_capacity(self.domain.len() + 1);
        for label in self.domain.split('.').rev() {
            if label.is_empty() || label.contains('\0') {
                return SignerError::invalid_input_err("invalid TON app domain");
            }
            encoded.extend_from_slice(label.as_bytes());
            encoded.push(0);
        }
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::{BagOfCells, CellBuilder, testkit::TEST_ADDRESS};

    fn sample_cell() -> String {
        let mut builder = CellBuilder::new();
        builder.store_u32(32, 0).unwrap();
        BagOfCells::from_root(builder.build().unwrap()).to_base64(true).unwrap()
    }

    #[test]
    fn test_ton_sign_message_data() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(payload.clone(), "example.com".to_string(), TEST_ADDRESS.to_string());

        let bytes = data.to_bytes();
        let parsed = TonSignMessageData::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.payload, payload);
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.address, TEST_ADDRESS);
    }

    #[test]
    fn test_build_sign_data_hash() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(payload, "example.com".to_string(), TEST_ADDRESS.to_string());

        let hash = data.hash(1234567890).unwrap();

        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_build_sign_data_hash_cell() {
        let payload = TonSignDataPayload::Cell {
            schema: "comment#00000000 text:SnakeData = InMsgBody;".to_string(),
            cell: sample_cell(),
        };
        let data = TonSignMessageData::new(payload, "example.com".to_string(), TEST_ADDRESS.to_string());
        let hash = data.hash(1234567890).unwrap();

        assert_eq!(hex::encode(hash), "6ad868b3019bdbd16bc89eecae337bcfcfab02bcb86432cd0cdbc829dfb49adb");
    }

    #[test]
    fn test_build_sign_data_hash_accepts_raw_address() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(
            payload,
            "example.com".to_string(),
            "0:58d5c54fbb8488af7eaad0cdc759ca8f6ff79fc9555106c1339b037ec0a40347".to_string(),
        );

        let hash = data.hash(1234567890).unwrap();

        assert_eq!(hash.len(), 32);
    }
}
