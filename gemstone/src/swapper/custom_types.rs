use primitives::{Chain, ChainType, SwapProvider, SwapQuote, SwapQuoteProtocolRequest};
use serde_json;
use std::str::FromStr;

uniffi::custom_type!(SwapProvider, String);
uniffi::custom_type!(Chain, String);
uniffi::custom_type!(ChainType, String);
uniffi::custom_type!(SwapQuote, String);
uniffi::custom_type!(SwapQuoteProtocolRequest, String);

impl crate::UniffiCustomTypeConverter for SwapProvider {
    type Builtin = String;

    fn into_custom(name: Self::Builtin) -> uniffi::Result<Self> {
        Ok(SwapProvider { name })
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.name
    }
}

impl crate::UniffiCustomTypeConverter for Chain {
    type Builtin = String;

    fn into_custom(chain: Self::Builtin) -> uniffi::Result<Self> {
        Chain::from_str(&chain).map_err(anyhow::Error::msg)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.as_ref().to_string()
    }
}

// FIXME add macro for serde json conversion
impl crate::UniffiCustomTypeConverter for ChainType {
    type Builtin = String;

    fn into_custom(json_string: Self::Builtin) -> uniffi::Result<Self> {
        let obj: ChainType = serde_json::from_str(&json_string).map_err(anyhow::Error::msg)?;
        Ok(obj)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        serde_json::to_string(&obj).unwrap()
    }
}

impl crate::UniffiCustomTypeConverter for SwapQuote {
    type Builtin = String;

    fn into_custom(json_string: Self::Builtin) -> uniffi::Result<Self> {
        let obj: SwapQuote = serde_json::from_str(&json_string).map_err(anyhow::Error::msg)?;
        Ok(obj)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        serde_json::to_string(&obj).unwrap()
    }
}

impl crate::UniffiCustomTypeConverter for SwapQuoteProtocolRequest {
    type Builtin = String;

    fn into_custom(json_string: Self::Builtin) -> uniffi::Result<Self> {
        let obj: SwapQuoteProtocolRequest = serde_json::from_str(&json_string).map_err(anyhow::Error::msg)?;
        Ok(obj)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        serde_json::to_string(&obj).unwrap()
    }
}
