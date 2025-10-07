use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::thorchain::contracts::RouterInterface;
use primitives::{Chain, swap::ApprovalData};

use super::{
    DEFAULT_DEPOSIT_GAS_LIMIT, QUOTE_INTERVAL, QUOTE_MINIMUM, QUOTE_QUANTITY, ThorChain, asset::THORChainAsset, chain::THORChainName, memo::ThorchainMemo,
    model::RouteData,
};
use crate::{
    FetchQuoteData, Swapper, SwapperChainAsset, SwapperError, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData, SwapperQuoteRequest,
    SwapperRoute, SwapperSwapResult, approval::check_approval_erc20, asset::*,
};

const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[async_trait]
impl<C> Swapper for ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn provider(&self) -> &SwapperProviderType {
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
                    vec![
                        ETHEREUM_USDT.id.clone(),
                        ETHEREUM_USDC.id.clone(),
                        ETHEREUM_DAI.id.clone(),
                        ETHEREUM_WBTC.id.clone(),
                    ],
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

    async fn fetch_quote(&self, request: &SwapperQuoteRequest) -> Result<SwapperQuote, SwapperError> {
        let from_asset = THORChainAsset::from_asset_id(&request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(&request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?;

        let value = self.value_from(request.clone().value, from_asset.decimals as i32);

        // thorchain is not included in inbound addresses
        if from_asset.chain != THORChainName::Thorchain {
            // min fee validation
            let inbound_addresses = self.swap_client.get_inbound_addresses().await?;
            let from_inbound_address = &inbound_addresses
                .iter()
                .find(|address| address.chain == from_asset.chain.long_name())
                .ok_or(SwapperError::InvalidRoute)?;

            if from_inbound_address.dust_threshold > value {
                return Err(SwapperError::InputAmountTooSmall);
            }

            // if (from_inbound_address.outbound_fee.clone() * from_inbound_address.gas_rate.clone()) > value {
            //     return Err(SwapperError::InputAmountTooSmall);
            // }
        }

        let fee = request.options.clone().fee.unwrap_or_default().thorchain;

        let quote = self
            .swap_client
            .get_quote(
                from_asset.clone(),
                to_asset.clone(),
                value.to_string(),
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps.into(),
            )
            .await?;

        let to_value = self.value_to(quote.expected_amount_out, to_asset.decimals as i32);

        let route_data = RouteData {
            router_address: quote.router.clone(),
            inbound_address: quote.inbound_address.clone(),
        };

        let quote = SwapperQuote {
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            data: SwapperProviderData {
                provider: self.provider().clone(),
                routes: vec![SwapperRoute {
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

    async fn fetch_quote_data(&self, quote: &SwapperQuote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
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
                let from_amount: U256 = value.to_string().parse().map_err(SwapperError::from)?;
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    from_asset.token_id.clone().unwrap(),
                    route_data.router_address.clone().unwrap(),
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
        let gas_limit = if approval.is_some() {
            Some(DEFAULT_DEPOSIT_GAS_LIMIT.to_string())
        } else {
            None
        };

        let data = if from_asset.use_evm_router() {
            // only used for swapping from ERC20 tokens
            let to = route_data.router_address.clone().unwrap();
            let inbound_address = Address::from_str(&route_data.inbound_address.unwrap_or_default()).unwrap();
            let token_address = Address::from_str(&quote.request.from_asset.asset_id().token_id.clone().unwrap()).unwrap();
            let amount = U256::from_str(&value).unwrap();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 86400; // + 1 day
            let expiry = U256::from_str(timestamp.to_string().as_str()).unwrap();

            let call = RouterInterface::depositWithExpiryCall {
                inbound_address,
                token_address,
                amount,
                memo,
                expiry,
            }
            .abi_encode();

            SwapperQuoteData {
                to,
                value: "0".to_string(),
                data: HexEncode(call.clone()),
                approval,
                gas_limit,
            }
        } else {
            SwapperQuoteData {
                to: route_data.inbound_address.unwrap_or_default(),
                value,
                data: self.data(from_asset.chain, memo),
                approval,
                gas_limit,
            }
        };

        Ok(data)
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapperSwapResult, SwapperError> {
        let status = self.swap_client.get_transaction_status(transaction_hash).await?;

        let swap_status = status.observed_tx.swap_status();
        let memo_parsed = ThorchainMemo::parse(&status.tx.memo);
        let destination_chain = memo_parsed.as_ref().and_then(|m| m.destination_chain());

        // Extract the first non-zero destination transaction hash from out_hashes
        let destination_tx_hash = if let Some(out_hashes) = &status.observed_tx.out_hashes {
            out_hashes.iter().find(|hash| *hash != ZERO_HASH && !hash.is_empty()).cloned()
        } else {
            None
        };

        let (to_chain, to_tx_hash) = match (destination_chain, destination_tx_hash) {
            (Some(dest_chain), Some(dest_hash)) => (Some(dest_chain), Some(dest_hash)),
            (Some(dest_chain), None) => (Some(dest_chain), None),
            _ => (None, None),
        };

        Ok(SwapperSwapResult {
            status: swap_status,
            from_chain: chain,
            from_tx_hash: transaction_hash.to_string(),
            to_chain,
            to_tx_hash,
        })
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{alien::reqwest_provider::NativeProvider, testkit::mock_quote};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_thorchain_swap_trx_to_bnb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let request = mock_quote(Chain::Tron.as_asset_id(), Chain::SmartChain.as_asset_id());

        let quote = swapper.fetch_quote(&request).await?;

        println!("quote: {:#?}", quote);

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert!(quote.eta_in_seconds.is_some());
        assert!(!quote.data.routes.is_empty());

        Ok(())
    }
}
