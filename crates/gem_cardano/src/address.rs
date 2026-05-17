use primitives::SignerError;

const MAINNET_NETWORK_ID: u8 = 1;
const BASE_KEY_ADDRESS_TYPE: u8 = 0;
const ENTERPRISE_KEY_ADDRESS_TYPE: u8 = 6;
const ADDRESS_HEADER_LENGTH: usize = 1;
const KEY_HASH_LENGTH: usize = 28;
const ENTERPRISE_ADDRESS_LENGTH: usize = ADDRESS_HEADER_LENGTH + KEY_HASH_LENGTH;
const BASE_ADDRESS_LENGTH: usize = ADDRESS_HEADER_LENGTH + KEY_HASH_LENGTH + KEY_HASH_LENGTH;

#[derive(Debug)]
pub(crate) struct ShelleyAddress {
    bytes: Vec<u8>,
}

impl ShelleyAddress {
    pub(crate) fn parse(address: &str) -> Result<Self, SignerError> {
        let (hrp, bytes) = bech32::decode(address).map_err(|_| SignerError::invalid_input("invalid Cardano address"))?;
        if hrp.as_str() != "addr" {
            return SignerError::invalid_input_err("unsupported Cardano address network");
        }

        Self::from_bytes(bytes)
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn payment_hash(&self) -> &[u8] {
        &self.bytes[ADDRESS_HEADER_LENGTH..ADDRESS_HEADER_LENGTH + KEY_HASH_LENGTH]
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, SignerError> {
        if bytes.is_empty() {
            return SignerError::invalid_input_err("invalid Cardano address");
        }
        let address = Self { bytes };
        if address.network_id() != MAINNET_NETWORK_ID {
            return SignerError::invalid_input_err("unsupported Cardano address network");
        }
        match address.address_type() {
            BASE_KEY_ADDRESS_TYPE => {
                if address.bytes.len() != BASE_ADDRESS_LENGTH {
                    return SignerError::invalid_input_err("invalid Cardano base address");
                }
            }
            ENTERPRISE_KEY_ADDRESS_TYPE => {
                if address.bytes.len() != ENTERPRISE_ADDRESS_LENGTH {
                    return SignerError::invalid_input_err("invalid Cardano enterprise address");
                }
            }
            _ => return SignerError::invalid_input_err("unsupported Cardano address type"),
        }
        Ok(address)
    }

    fn address_type(&self) -> u8 {
        self.bytes[0] >> 4
    }

    fn network_id(&self) -> u8 {
        self.bytes[0] & 0x0f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelley_address_decode() {
        let address = ShelleyAddress::parse("addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23").unwrap();
        assert_eq!(
            hex::encode(address.as_bytes()),
            "01df58ee97ce7a46cd8bdeec4e5f3a03297eb197825ed5681191110804df22424b6880b39e4bac8c58de9fe6d23d79aaf44756389d827aa09b"
        );
        assert_eq!(hex::encode(address.payment_hash()), "df58ee97ce7a46cd8bdeec4e5f3a03297eb197825ed5681191110804");
    }

    #[test]
    fn test_shelley_address_rejects_unsupported() {
        assert!(ShelleyAddress::parse("stake1uykptcz226y5r5at5rfqqm00p9n0z0yfajz3gk3j3wm8dxg2sn0r4").is_err());
        assert!(ShelleyAddress::parse("addr_test1qr4p6f6mm0q9kfyyd9u30umk9cc6gk0nxu25k5rsc4fp7ls7k0qqxslcwwj4gvn4yfmdyrfgwjt3ztuz4zpy4242u0m95r0n").is_err());
    }
}
