mod asset;
mod bigint;
mod chain;
mod client;
mod constants;
mod memo;
pub(crate) mod model;
mod provider;
mod quote_data_mapper;

use primitives::Chain;
use std::sync::Arc;

use crate::alien::RpcProvider;
use gem_client::Client;

use super::{ProviderType, SwapperError, SwapperProvider};

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;
const OUTBOUND_DELAY_SECONDS: u32 = 60;

// FIXME: estimate gas limit with memo x bytes
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 90_000;

#[derive(Debug)]
pub struct ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub provider: ProviderType,
    pub rpc_provider: Arc<dyn RpcProvider>,
    pub(crate) swap_client: client::ThorChainSwapClient<C>,
}

impl<C> ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub fn with_client(swap_client: client::ThorChainSwapClient<C>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Thorchain),
            rpc_provider,
            swap_client,
        }
    }

    fn get_eta_in_seconds(&self, destination_chain: Chain, total_swap_seconds: Option<u32>) -> u32 {
        destination_chain.block_time() / 1000 + OUTBOUND_DELAY_SECONDS + total_swap_seconds.unwrap_or(0)
    }

    fn map_quote_error(&self, error: SwapperError, decimals: i32) -> SwapperError {
        match error {
            SwapperError::InputAmountError { min_amount: Some(min) } => SwapperError::InputAmountError {
                min_amount: Some(self.value_to(min, decimals).to_string()),
            },
            other => other,
        }
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::alien::reqwest_provider::NativeProvider;
    use std::sync::Arc;

    #[test]
    fn test_value_from() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));

        let value = "1000000000".to_string();

        let result = thorchain.value_from(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("0").unwrap());

        let result = thorchain.value_from(value.clone(), 10);
        assert_eq!(result, BigInt::from_str("10000000").unwrap());

        let result = thorchain.value_from(value.clone(), 6);
        assert_eq!(result, BigInt::from_str("100000000000").unwrap());

        let result = thorchain.value_from(value.clone(), 8);
        assert_eq!(result, BigInt::from(1000000000));
    }

    #[test]
    fn test_value_to() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));

        let value = "10000000".to_string();

        let result = thorchain.value_to(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("100000000000000000").unwrap());

        let result = thorchain.value_to(value.clone(), 10);
        assert_eq!(result, BigInt::from(1000000000));

        let result = thorchain.value_to(value.clone(), 6);
        assert_eq!(result, BigInt::from(100000));

        let result = thorchain.value_to(value.clone(), 8);
        assert_eq!(result, BigInt::from(10000000));
    }

    #[test]
    fn test_get_eta_in_seconds() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));

        let eta = thorchain.get_eta_in_seconds(Chain::Bitcoin, None);
        assert_eq!(eta, 660);

        let eta = thorchain.get_eta_in_seconds(Chain::Bitcoin, Some(1200));
        assert_eq!(eta, 1860);

        let eta = thorchain.get_eta_in_seconds(Chain::SmartChain, Some(648));
        assert_eq!(eta, 709);
    }

    #[test]
    fn test_map_quote_error() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));

        let cases = vec![(18, "6614750000000000"), (8, "661475"), (6, "6614")];

        for (decimals, expected) in cases {
            let error = SwapperError::InputAmountError {
                min_amount: Some("661475".to_string()),
            };
            let result = thorchain.map_quote_error(error, decimals);
            assert_eq!(
                result,
                SwapperError::InputAmountError {
                    min_amount: Some(expected.to_string())
                }
            );
        }

        let error = SwapperError::InputAmountError { min_amount: None };
        assert_eq!(thorchain.map_quote_error(error, 18), SwapperError::InputAmountError { min_amount: None });

        let error = SwapperError::NotSupportedAsset;
        assert_eq!(thorchain.map_quote_error(error, 18), SwapperError::NotSupportedAsset);
    }
}
