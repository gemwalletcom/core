use primitives::{AssetId, Chain, ChainType};
use std::str::FromStr;

uniffi::custom_type!(Chain, String);
uniffi::custom_type!(ChainType, String);
uniffi::custom_type!(AssetId, String);

impl crate::UniffiCustomTypeConverter for Chain {
    type Builtin = String;

    fn into_custom(chain: Self::Builtin) -> uniffi::Result<Self> {
        Chain::from_str(&chain).map_err(anyhow::Error::msg)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.as_ref().to_string()
    }
}

impl crate::UniffiCustomTypeConverter for ChainType {
    type Builtin = String;

    fn into_custom(string: Self::Builtin) -> uniffi::Result<Self> {
        ChainType::from_str(&string).map_err(anyhow::Error::msg)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.as_ref().to_string()
    }
}

impl crate::UniffiCustomTypeConverter for AssetId {
    type Builtin = String;

    fn into_custom(string: Self::Builtin) -> uniffi::Result<Self> {
        AssetId::new(&string).ok_or(anyhow::anyhow!("invalid asset id"))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}
