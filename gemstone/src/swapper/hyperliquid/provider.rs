use std::{
    convert::TryFrom,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use gem_hypercore::core::{
    actions::{user::spot_send::SpotSend, HYPERCORE_EVM_BRIDGE_ADDRESS},
    hypercore::transfer_to_hyper_evm_typed_data,
};
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::{AssetId, Chain};
use serde::Serialize;

use crate::{
    network::AlienProvider,
    swapper::{
        asset::{HYPERCORE_HYPE, HYPERLIQUID_HYPE},
        FetchQuoteData, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData,
        SwapperQuoteRequest, SwapperRoute,
    },
};

const HYPERCORE_HYPE_TOKEN: &str = "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec";
const HYPERCORE_NATIVE_TRANSFER_GAS_LIMIT: &str = "21000";

#[derive(Debug)]
pub struct HyperliquidBridge {
    provider: SwapperProviderType,
}

impl Default for HyperliquidBridge {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::Hyperliquid),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HyperliquidBridgeDirection {
    CoreToEvm,
    EvmToCore,
}

#[derive(Debug, Serialize)]
struct HyperliquidRouteData {
    direction: HyperliquidRouteDirection,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum HyperliquidRouteDirection {
    CoreToEvm,
    EvmToCore,
}

impl From<HyperliquidBridgeDirection> for HyperliquidRouteDirection {
    fn from(direction: HyperliquidBridgeDirection) -> Self {
        match direction {
            HyperliquidBridgeDirection::CoreToEvm => Self::CoreToEvm,
            HyperliquidBridgeDirection::EvmToCore => Self::EvmToCore,
        }
    }
}

#[derive(Serialize)]
struct HypercoreTransferPayload {
    action: SpotSend,
    typed_data: String,
}

impl HyperliquidBridge {
    fn detect_direction(request: &SwapperQuoteRequest) -> Result<HyperliquidBridgeDirection, SwapperError> {
        let from_asset_id = request.from_asset.asset_id();
        let to_asset_id = request.to_asset.asset_id();

        Self::ensure_native(&from_asset_id)?;
        Self::ensure_native(&to_asset_id)?;

        match (from_asset_id.chain, to_asset_id.chain) {
            (Chain::HyperCore, Chain::Hyperliquid) => Ok(HyperliquidBridgeDirection::CoreToEvm),
            (Chain::Hyperliquid, Chain::HyperCore) => Ok(HyperliquidBridgeDirection::EvmToCore),
            _ => Err(SwapperError::NotSupportedPair),
        }
    }

    fn ensure_native(asset_id: &AssetId) -> Result<(), SwapperError> {
        if asset_id.is_native() {
            Ok(())
        } else {
            Err(SwapperError::NotSupportedAsset)
        }
    }

    fn current_time_millis() -> Result<u64, SwapperError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SwapperError::ComputeQuoteError("system time is before unix epoch".into()))?;
        u64::try_from(now.as_millis()).map_err(|_| SwapperError::ComputeQuoteError("timestamp overflow".into()))
    }

    fn format_amount(value: &str, decimals: u32) -> Result<String, SwapperError> {
        let amount = BigUint::from_str(value)?;

        if decimals == 0 {
            return Ok(amount.to_string());
        }

        let base = BigUint::from(10u32).pow(decimals);
        let integer = &amount / &base;
        let remainder = &amount % &base;

        if remainder.is_zero() {
            return Ok(integer.to_string());
        }

        let mut fractional = remainder.to_str_radix(10);
        let expected_len = decimals as usize;
        if fractional.len() < expected_len {
            let mut padded = String::with_capacity(expected_len);
            for _ in 0..(expected_len - fractional.len()) {
                padded.push('0');
            }
            padded.push_str(&fractional);
            fractional = padded;
        }
        let fractional_trimmed = fractional.trim_end_matches('0');

        Ok(format!("{}.{fractional_trimmed}", integer.to_string()))
    }
}

#[async_trait]
impl Swapper for HyperliquidBridge {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(Chain::HyperCore, vec![HYPERCORE_HYPE.id.clone()]),
            SwapperChainAsset::Assets(Chain::Hyperliquid, vec![HYPERLIQUID_HYPE.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapperQuoteRequest, _provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError> {
        let direction = Self::detect_direction(request)?;

        let route_data = HyperliquidRouteData { direction: direction.into() };

        let quote = SwapperQuote {
            from_value: request.value.clone(),
            to_value: request.value.clone(),
            data: SwapperProviderData {
                provider: self.provider.clone(),
                slippage_bps: 0,
                routes: vec![SwapperRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data)?,
                    gas_limit: None,
                }],
            },
            request: request.clone(),
            eta_in_seconds: None,
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapperQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let direction = Self::detect_direction(&quote.request)?;

        match direction {
            HyperliquidBridgeDirection::CoreToEvm => {
                let amount = Self::format_amount(&quote.request.value, quote.request.from_asset.decimals)?;
                let timestamp = Self::current_time_millis()?;

                let spot_send = SpotSend::new(amount, HYPERCORE_EVM_BRIDGE_ADDRESS.to_string(), timestamp, HYPERCORE_HYPE_TOKEN.to_string());
                let typed_data = transfer_to_hyper_evm_typed_data(spot_send.clone());
                let payload = HypercoreTransferPayload { action: spot_send, typed_data };

                Ok(SwapperQuoteData {
                    to: HYPERCORE_EVM_BRIDGE_ADDRESS.to_string(),
                    value: quote.request.value.clone(),
                    data: serde_json::to_string(&payload)?,
                    approval: None,
                    gas_limit: None,
                })
            }
            HyperliquidBridgeDirection::EvmToCore => Ok(SwapperQuoteData {
                to: HYPERCORE_EVM_BRIDGE_ADDRESS.to_string(),
                value: quote.request.value.clone(),
                // HYPE is the native HyperEVM gas token, so bridging back to HyperCore
                // only requires sending value to the system address.
                data: "0x".to_string(),
                approval: None,
                gas_limit: Some(HYPERCORE_NATIVE_TRANSFER_GAS_LIMIT.to_string()),
            }),
        }
    }
}
