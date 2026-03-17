use crate::address::ethereum_address_checksum;

pub fn deserialize_ethereum_address_checksum<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let address: String = serde::Deserialize::deserialize(deserializer)?;
    ethereum_address_checksum(&address).map_err(serde::de::Error::custom)
}
