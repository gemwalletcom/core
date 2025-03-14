use num_bigint::BigUint;
use num_traits::Num;

use serde::{de, Deserialize};

pub fn serialize_biguint<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_biguint_from_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    s.parse::<BigUint>().map_err(de::Error::custom)
}

pub fn deserialize_biguint_from_hex_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    BigUint::from_str_radix(&s[2..], 16).map_err(serde::de::Error::custom)
}

pub fn deserialize_biguint_from_option_hex_str<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => match BigUint::from_str_radix(&s[2..], 16) {
            Ok(biguint) => Ok(Some(biguint)),
            Err(e) => Err(serde::de::Error::custom(e)),
        },
        None => Ok(None),
    }
}
