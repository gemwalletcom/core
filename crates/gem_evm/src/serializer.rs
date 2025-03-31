use alloy_primitives::Address;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// Serializes an Address into a string with 0x prefix
pub fn serialize_address<S>(address: &Address, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&address.to_checksum(None))
}

/// Deserializes a string with 0x prefix into an Address
pub fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let address_str = String::deserialize(deserializer)?;
    Address::from_str(&address_str).map_err(|e| DeError::custom(format!("Invalid Ethereum address: {}", e)))
}

/// Serializes an Option<Address> into an Option<String> with 0x prefix
pub fn serialize_optional_address<S>(address: &Option<Address>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match address {
        Some(addr) => serializer.serialize_some(&addr.to_checksum(None)),
        None => serializer.serialize_none(),
    }
}

/// Deserializes an Option<String> with 0x prefix into an Option<Address>
pub fn deserialize_optional_address<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let result = Option::<String>::deserialize(deserializer)?;
    match result {
        Some(address_str) => {
            let address = Address::from_str(&address_str).map_err(|e| DeError::custom(format!("Invalid Ethereum address: {}", e)))?;
            Ok(Some(address))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(serialize_with = "serialize_address", deserialize_with = "deserialize_address")]
        address: Address,

        #[serde(serialize_with = "serialize_optional_address", deserialize_with = "deserialize_optional_address")]
        optional_address: Option<Address>,
    }

    #[test]
    fn test_address_serialization() {
        let address = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let optional_address = Some(Address::from_str("0xabcdef0123456789abcdef0123456789abcdef01").unwrap());

        let test_struct = TestStruct { address, optional_address };

        let serialized = serde_json::to_string(&test_struct).unwrap();

        let deserialized: TestStruct = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.address, address);
        assert_eq!(deserialized.optional_address, optional_address);
    }
}
