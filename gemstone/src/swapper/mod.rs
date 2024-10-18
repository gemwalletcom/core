use crate::network::AlienProvider;
use async_trait::async_trait;
use primitives::{Chain, ChainType, SwapProvider, SwapQuote, SwapQuoteProtocolRequest};
use std::{fmt::Debug, str::FromStr, sync::Arc};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemSwapperTrait: Send + Sync + Debug {
    async fn fetch_quote(&self, request: SwapQuoteProtocolRequest) -> SwapQuote;
}

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    pub network_provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    fn new(network_provider: Arc<dyn AlienProvider>) -> Self {
        Self { network_provider }
    }
}

#[uniffi::export]
#[async_trait]
impl GemSwapperTrait for GemSwapper {
    async fn fetch_quote(&self, _request: SwapQuoteProtocolRequest) -> SwapQuote {
        // TODO: Implement
        SwapQuote {
            chain_type: ChainType::Ethereum,
            from_amount: "0.0".to_string(),
            to_amount: "0.0".to_string(),
            fee_percent: 0.0,
            provider: SwapProvider { name: "1inch".to_string() },
            data: None,
            approval: None,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, SwapQuoteProtocolRequest};
    use serde_json;
    #[test]
    fn test_swap_provider() {
        let request = SwapQuoteProtocolRequest {
            from_asset: AssetId::from(Chain::Ethereum, None),
            to_asset: AssetId::from(Chain::Ethereum, None),
            wallet_address: String::from("0x1234567890abcdef"),
            destination_address: String::from("0x1234567890abcdef"),
            amount: String::from("0.0"),
            mode: primitives::SwapMode::ExactIn,
            include_data: false,
        };

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            json,
            r#"{"fromAsset":{"chain":"ethereum","tokenId":null},"toAsset":{"chain":"ethereum","tokenId":null},"walletAddress":"0x1234567890abcdef","destinationAddress":"0x1234567890abcdef","amount":"0.0","mode":"exactin","includeData":false}"#
        );
    }
}
