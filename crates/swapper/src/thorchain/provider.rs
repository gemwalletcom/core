use std::collections::HashSet;
use std::sync::Arc;

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, Transaction, TransactionType, hex::decode_hex_utf8, swap::ApprovalData};

use num_bigint::BigInt;

use super::{
    DUST_THRESHOLD_MULTIPLIER, QUOTE_INTERVAL, QUOTE_MINIMUM, QUOTE_QUANTITY, ThorChain,
    asset::{THORChainAsset, value_to},
    chain::THORChainName,
    memo::ThorchainMemo,
    model::RouteData,
    quote_data_mapper, swap_mapper,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData, approval::check_approval_erc20, asset::*, thorchain::client::ThorChainSwapClient,
};

pub struct ThorchainCrossChain;

impl ThorchainCrossChain {
    fn router_address(chain: &Chain) -> Option<&'static str> {
        match chain {
            Chain::Ethereum => Some("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146"),
            Chain::SmartChain => Some("0xb30eC53F98ff5947EDe720D32aC2da7e52A5f56b"),
            Chain::AvalancheC => Some("0x8F66c4AE756BEbC49Ec8B81966DD8bba9f127549"),
            Chain::Base => Some("0x68208D99746b805a1Ae41421950A47b711E35681"),
            _ => None,
        }
    }

    pub fn static_router_addresses() -> Vec<&'static str> {
        Chain::all().iter().filter_map(|chain| Self::router_address(chain)).collect()
    }

    fn has_swap_memo(transaction: &primitives::Transaction) -> bool {
        if transaction.memo.as_deref().is_some_and(ThorchainMemo::is_swap) {
            return true;
        }
        if let Some(decoded) = transaction.data.as_deref().and_then(decode_hex_utf8)
            && ThorchainMemo::is_swap(&decoded)
        {
            return true;
        }
        false
    }
}

impl crate::cross_chain::CrossChainProvider for ThorchainCrossChain {
    fn provider(&self) -> SwapperProvider {
        SwapperProvider::Thorchain
    }

    fn is_swap(&self, transaction: &Transaction) -> bool {
        if Self::has_swap_memo(transaction) {
            return true;
        }
        if let Some(router) = Self::router_address(&transaction.asset_id.chain) {
            return router == transaction.to && transaction.transaction_type == TransactionType::Transfer;
        }
        false
    }
}

impl ThorChain<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Thorchain).expect("Failed to get Thorchain endpoint");
        let swap_client = ThorChainSwapClient::new(RpcClient::new(endpoint, rpc_provider.clone()));
        Self::with_client(swap_client, rpc_provider)
    }
}

#[async_trait]
impl<C> Swapper for ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        Chain::all()
            .into_iter()
            .filter_map(|chain| THORChainName::from_chain(&chain).map(|name| name.chain()))
            .collect::<Vec<Chain>>()
            .into_iter()
            .map(|chain| match chain {
                Chain::Ethereum => SwapperChainAsset::Assets(
                    chain,
                    vec![ETHEREUM_USDT.id.clone(), ETHEREUM_USDC.id.clone(), ETHEREUM_DAI.id.clone(), ETHEREUM_WBTC.id.clone()],
                ),
                Chain::Thorchain => SwapperChainAsset::Assets(chain, vec![THORCHAIN_TCY.id.clone()]),
                Chain::SmartChain => SwapperChainAsset::Assets(chain, vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone()]),
                Chain::AvalancheC => SwapperChainAsset::Assets(chain, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
                Chain::Base => SwapperChainAsset::Assets(chain, vec![BASE_USDC.id.clone(), BASE_CBBTC.id.clone()]),
                Chain::Tron => SwapperChainAsset::Assets(chain, vec![TRON_USDT.id.clone()]),
                _ => SwapperChainAsset::Assets(chain, vec![]),
            })
            .collect()
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<Vec<String>, SwapperError> {
        let inbound = self.client.get_inbound_addresses().await?;
        let addresses = inbound.iter().flat_map(|entry| {
            let chain = THORChainName::from_symbol(&entry.chain);
            let checksum = move |addr: String| chain.as_ref().map(|c| c.checksum_address(&addr)).unwrap_or(addr);
            let addr = std::iter::once(checksum(entry.address.clone()));
            let router = entry.router.iter().filter(|r| !r.is_empty()).map(move |r| checksum(r.clone()));
            addr.chain(router)
        });
        let static_addresses = ThorchainCrossChain::static_router_addresses().into_iter().map(String::from);
        Ok(addresses.chain(static_addresses).collect::<HashSet<_>>().into_iter().collect())
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_asset = THORChainAsset::from_asset_id(&request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(&request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?;

        let value = super::asset::value_from(&request.value, from_asset.decimals as i32);

        if from_asset.chain != THORChainName::Thorchain {
            let inbound_addresses = self.client.get_inbound_addresses().await?;
            let from_inbound_address = inbound_addresses
                .iter()
                .find(|address| address.chain == from_asset.chain.long_name())
                .ok_or(SwapperError::InvalidRoute)?;

            let min_value = min_value(&from_inbound_address.dust_threshold);
            if min_value > value {
                return Err(SwapperError::InputAmountError {
                    min_amount: Some(value_to(&min_value.to_string(), from_asset.decimals as i32).to_string()),
                });
            }
        }

        let fee = request.options.clone().fee.unwrap_or_default().thorchain;

        let quote = self
            .client
            .get_quote(
                from_asset.clone(),
                to_asset.clone(),
                value.to_string(),
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps.into(),
            )
            .await
            .map_err(|e| self.map_quote_error(e, from_asset.decimals as i32))?;

        let to_value = super::asset::value_to(&quote.expected_amount_out, to_asset.decimals as i32);
        let inbound_address = RouteData::get_inbound_address(&from_asset, quote.inbound_address.clone())?;
        let route_data = RouteData {
            router_address: quote.router.clone(),
            inbound_address,
        };

        let quote = Quote {
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data).unwrap_or_default(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(self.get_eta_in_seconds(request.to_asset.chain(), quote.total_swap_seconds)),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let fee = quote.request.options.clone().fee.unwrap_or_default().thorchain;
        let from_asset = THORChainAsset::from_asset_id(&quote.request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(&quote.request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?;

        let memo = to_asset
            .get_memo(
                quote.request.destination_address.clone(),
                QUOTE_MINIMUM,
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps,
            )
            .unwrap();

        let route_data: RouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let value = quote.request.value.clone();

        let approval: Option<ApprovalData> = {
            if from_asset.use_evm_router() {
                let router_address = route_data.router_address.clone().ok_or(SwapperError::InvalidRoute)?;
                let from_amount: U256 = value.to_string().parse().map_err(SwapperError::from)?;
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    from_asset.token_id.clone().unwrap(),
                    router_address,
                    from_amount,
                    self.rpc_provider.clone(),
                    &from_asset.chain.chain(),
                )
                .await?
                .approval_data()
            } else {
                None
            }
        };

        let data = quote_data_mapper::map_quote_data(&from_asset, &route_data, quote.request.from_asset.asset_id().token_id.clone(), value, memo, approval);

        Ok(data)
    }

    async fn get_swap_result(&self, _chain: Chain, hash: &str) -> Result<SwapResult, SwapperError> {
        let hash = hash.strip_prefix("0x").unwrap_or(hash).to_uppercase();
        let response = self.client.get_transaction_status(&hash).await?;
        Ok(swap_mapper::map_swap_result(&response))
    }
}

fn min_value(dust_threshold: &BigInt) -> BigInt {
    dust_threshold * DUST_THRESHOLD_MULTIPLIER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_value() {
        assert_eq!(min_value(&BigInt::from(10000)), BigInt::from(20000));
        assert_eq!(min_value(&BigInt::from(0)), BigInt::from(0));
        assert_eq!(min_value(&BigInt::from(50000)), BigInt::from(100000));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, testkit::mock_quote};
    use primitives::swap::SwapStatus;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_thorchain_quote_trx_to_bnb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Tron.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::SmartChain.as_asset_id());
        let request = mock_quote(from_asset, to_asset);

        let quote = swapper.fetch_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(quote.eta_in_seconds.is_some());
        assert!(!quote.data.routes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_thorchain_quote_rune_to_cosmos() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Thorchain.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::Cosmos.as_asset_id());
        let mut request = mock_quote(from_asset, to_asset);
        request.value = "100000000".to_string();

        let quote = swapper.fetch_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(quote.eta_in_seconds.is_some());
        assert!(!quote.data.routes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_thorchain_quote_rejects_below_min_value() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Xrp.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::Thorchain.as_asset_id());
        let mut request = mock_quote(from_asset, to_asset);
        request.value = "1".to_string();

        let err = swapper.fetch_quote(&request).await.expect_err("expected error");
        assert!(matches!(err, SwapperError::InputAmountError { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn test_thorchain_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let tx_hash = "324c16cf014cceca1b2e1c078417f736c9833197735b71a4e875bbb3b07b2fe4";
        let result = swapper.get_swap_result(Chain::Doge, tx_hash).await?;

        assert_eq!(result.status, SwapStatus::Completed);

        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, Chain::Doge.as_asset_id());
        assert!(!metadata.from_value.is_empty());
        assert!(!metadata.to_value.is_empty());
        assert_eq!(metadata.provider.unwrap(), "thorchain");

        Ok(())
    }
}
