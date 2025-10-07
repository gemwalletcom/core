mod asset;
mod chain;
mod client;
mod default;
mod memo;
mod model;
mod provider;

use chain::THORChainName;
use num_bigint::BigInt;
use primitives::Chain;
use std::{str::FromStr, sync::Arc};

use crate::alien::RpcProvider;
use gem_client::Client;

use super::{SwapperProvider, ProviderType};

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

    fn data(&self, chain: THORChainName, memo: String) -> String {
        if chain.is_evm_chain() {
            return hex::encode(memo.as_bytes());
        }
        memo
    }

    fn value_from(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow(decimals as u32)
        } else {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow(decimals.unsigned_abs())
        }
    }

    fn value_to(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow((decimals).unsigned_abs())
        } else {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow((decimals).unsigned_abs())
        }
    }

    fn get_eta_in_seconds(&self, destination_chain: Chain, total_swap_seconds: Option<u32>) -> u32 {
        destination_chain.block_time() / 1000 + OUTBOUND_DELAY_SECONDS + total_swap_seconds.unwrap_or(0)
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::alien::reqwest_provider::NativeProvider;
    use std::sync::Arc;

    #[test]
    fn test_data() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));
        let memo = "test".to_string();

        let result = thorchain.data(THORChainName::Ethereum, memo.clone());
        assert_eq!(result, hex::encode(memo.as_bytes()));

        let result = thorchain.data(THORChainName::Bitcoin, memo.clone());
        assert_eq!(result, memo);
    }

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
}
