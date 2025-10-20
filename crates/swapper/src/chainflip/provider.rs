use alloy_primitives::{U256, hex};
use async_trait::async_trait;
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::{fmt::Debug, str::FromStr, sync::Arc};

use super::{
    ChainflipRouteData,
    broker::{
        BrokerClient, ChainflipAsset, DcaParameters, RefundParameters, VaultSwapBtcExtras, VaultSwapEvmExtras, VaultSwapExtras, VaultSwapResponse,
        VaultSwapSolanaExtras,
    },
    capitalize::capitalize_first_letter,
    client::{ChainflipClient, QuoteRequest as ChainflipQuoteRequest, QuoteResponse},
    price::{apply_slippage, price_to_hex_price},
    seed::generate_random_seed,
    tx_builder,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    alien::RpcProvider,
    approval::check_approval_erc20,
    asset::{ARBITRUM_USDC, ETHEREUM_FLIP, ETHEREUM_USDC, ETHEREUM_USDT, SOLANA_USDC},
    config::DEFAULT_CHAINFLIP_FEE_BPS,
    slippage,
};
use primitives::{ChainType, chain::Chain, swap::QuoteAsset};

const DEFAULT_SWAP_ERC20_GAS_LIMIT: u64 = 100_000;

#[derive(Debug)]
pub struct ChainflipProvider<CX, BR>
where
    CX: Client + Clone + Send + Sync + Debug + 'static,
    BR: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    chainflip_client: ChainflipClient<CX>,
    broker_client: BrokerClient<BR>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl<CX, BR> ChainflipProvider<CX, BR>
where
    CX: Client + Clone + Send + Sync + Debug + 'static,
    BR: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_clients(chainflip_client: ChainflipClient<CX>, broker_client: BrokerClient<BR>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Chainflip),
            chainflip_client,
            broker_client,
            rpc_provider,
        }
    }

    fn map_asset_id(asset: &QuoteAsset) -> ChainflipAsset {
        let asset_id = asset.asset_id();
        let chain_name = capitalize_first_letter(asset_id.chain.as_ref());
        ChainflipAsset {
            chain: chain_name,
            asset: asset.symbol.clone(),
        }
    }

    fn map_chainflip_chain_to_chain(chainflip_chain: &str) -> Option<Chain> {
        Chain::from_str(&chainflip_chain.to_lowercase()).ok()
    }
}

fn get_best_quote(mut quotes: Vec<QuoteResponse>, fee_bps: u32) -> (BigUint, u32, u32, ChainflipRouteData) {
    quotes.sort_by(|a, b| b.egress_amount.cmp(&a.egress_amount));
    let quote = &quotes[0];

    let (egress_amount, slippage_bps, eta_in_seconds, boost_fee, estimated_price, dca_parameters) = if let Some(boost_quote) = &quote.boost_quote {
        (
            boost_quote.egress_amount.clone(),
            boost_quote.slippage_bps(),
            boost_quote.estimated_duration_seconds as u32,
            Some(boost_quote.estimated_boost_fee_bps),
            boost_quote.estimated_price.clone(),
            boost_quote.dca_params.as_ref().map(|dca| DcaParameters {
                number_of_chunks: dca.number_of_chunks,
                chunk_interval: dca.chunk_interval_blocks,
            }),
        )
    } else {
        (
            quote.egress_amount.clone(),
            quote.slippage_bps(),
            quote.estimated_duration_seconds as u32,
            None,
            quote.estimated_price.clone(),
            quote.dca_params.as_ref().map(|dca| DcaParameters {
                number_of_chunks: dca.number_of_chunks,
                chunk_interval: dca.chunk_interval_blocks,
            }),
        )
    };

    (
        egress_amount,
        slippage_bps,
        eta_in_seconds,
        ChainflipRouteData {
            boost_fee,
            fee_bps,
            estimated_price,
            dca_parameters,
        },
    )
}

#[async_trait]
impl<CX, BR> Swapper for ChainflipProvider<CX, BR>
where
    CX: Client + Clone + Send + Sync + Debug + 'static,
    BR: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![
            SwapperChainAsset::Assets(Chain::Bitcoin, vec![]),
            SwapperChainAsset::Assets(
                Chain::Ethereum,
                vec![ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone(), ETHEREUM_FLIP.id.clone()],
            ),
            SwapperChainAsset::Assets(Chain::Solana, vec![SOLANA_USDC.id.clone()]),
            SwapperChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if request.from_asset.chain().chain_type() == ChainType::Bitcoin {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let src_asset = Self::map_asset_id(&request.from_asset);
        let dest_asset = Self::map_asset_id(&request.to_asset);

        let fee_bps = DEFAULT_CHAINFLIP_FEE_BPS;
        let quote_request = ChainflipQuoteRequest {
            amount: request.value.clone(),
            src_chain: src_asset.chain.clone(),
            src_asset: src_asset.asset.clone(),
            dest_chain: dest_asset.chain,
            dest_asset: dest_asset.asset,
            is_vault_swap: true,
            dca_enabled: true,
            broker_commission_bps: Some(fee_bps),
        };

        let quotes = self.chainflip_client.get_quote(&quote_request).await?;
        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, fee_bps);

        Ok(Quote {
            from_value: request.value.clone(),
            to_value: egress_amount.to_string(),
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data).unwrap(),
                    gas_limit: None,
                }],
            },
            eta_in_seconds: Some(eta_in_seconds),
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        let source_asset = Self::map_asset_id(&quote.request.from_asset);
        let destination_asset = Self::map_asset_id(&quote.request.to_asset);

        let input_amount: BigUint = quote.request.value.parse()?;

        let route_data: ChainflipRouteData = serde_json::from_str(&quote.data.routes[0].route_data)?;
        let chain = source_asset.chain.clone();
        let price = route_data
            .estimated_price
            .parse::<f64>()
            .map_err(|_| SwapperError::TransactionError("Invalid price".to_string()))?;
        let price_slippage = apply_slippage(price, quote.data.slippage_bps);
        let quote_asset_decimals = quote.request.to_asset.decimals;
        let base_asset_decimals = quote.request.from_asset.decimals;
        let min_price = price_to_hex_price(price_slippage, quote_asset_decimals, base_asset_decimals).map_err(SwapperError::TransactionError)?;
        let extra_params = if from_asset.chain.chain_type() == ChainType::Ethereum {
            VaultSwapExtras::Evm(VaultSwapEvmExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters: RefundParameters {
                    retry_duration: 150,
                    refund_address: quote.request.wallet_address.clone(),
                    min_price,
                },
            })
        } else if from_asset.chain.chain_type() == ChainType::Bitcoin {
            let output_amount: U256 = quote.to_value.parse()?;
            let min_output_amount = slippage::apply_slippage_in_bp(&output_amount, quote.data.slippage_bps);
            VaultSwapExtras::Bitcoin(VaultSwapBtcExtras {
                chain,
                min_output_amount: BigUint::from_bytes_le(&min_output_amount.to_le_bytes::<32>()),
                retry_duration: 6,
            })
        } else if from_asset.chain.chain_type() == ChainType::Solana {
            VaultSwapExtras::Solana(VaultSwapSolanaExtras {
                from: quote.request.wallet_address.clone(),
                seed: hex::encode_prefixed(generate_random_seed(32)),
                chain,
                input_amount: input_amount.to_u64().unwrap(),
                refund_parameters: RefundParameters {
                    retry_duration: 10,
                    refund_address: quote.request.wallet_address.clone(),
                    min_price,
                },
            })
        } else {
            VaultSwapExtras::None
        };

        let response = self
            .broker_client
            .encode_vault_swap(
                source_asset,
                destination_asset,
                quote.request.destination_address.clone(),
                route_data.fee_bps,
                route_data.boost_fee,
                extra_params,
                route_data.dca_parameters,
            )
            .await?;

        match response {
            VaultSwapResponse::Evm(response) => {
                let value = if from_asset.is_native() {
                    quote.request.value.clone()
                } else {
                    "0".to_string()
                };

                let approval = if from_asset.chain.chain_type() == ChainType::Ethereum && !from_asset.is_native() {
                    let approval = check_approval_erc20(
                        quote.request.wallet_address.clone(),
                        from_asset.token_id.unwrap(),
                        response.to.clone(),
                        U256::from_le_slice(&input_amount.to_bytes_le()),
                        self.rpc_provider.clone(),
                        &from_asset.chain,
                    )
                    .await?;
                    approval.approval_data()
                } else {
                    None
                };

                let gas_limit = if approval.is_some() {
                    Some(DEFAULT_SWAP_ERC20_GAS_LIMIT.to_string())
                } else {
                    None
                };

                Ok(SwapperQuoteData::new_contract(response.to, value, response.calldata, approval, gas_limit))
            }
            VaultSwapResponse::Bitcoin(response) => Ok(SwapperQuoteData::new_contract(
                response.deposit_address,
                quote.request.value.clone(),
                response.nulldata_payload,
                None,
                None,
            )),
            VaultSwapResponse::Solana(response) => {
                let data = tx_builder::build_solana_tx(&quote.request.wallet_address, &response, self.rpc_provider.clone())
                    .await
                    .map_err(SwapperError::TransactionError)?;
                Ok(SwapperQuoteData::new_contract(response.program_id, "".into(), data, None, None))
            }
        }
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let status = self.chainflip_client.get_tx_status(transaction_hash).await?;
        let swap_status = status.swap_status();
        let to_tx_hash = status.swap_egress.as_ref().and_then(|x| x.tx_ref.clone());

        Ok(SwapResult {
            status: swap_status,
            from_chain: chain,
            from_tx_hash: transaction_hash.to_string(),
            to_chain: Self::map_chainflip_chain_to_chain(&status.dest_chain),
            to_tx_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_best_quote() {
        let json = include_str!("./test/chainflip_quotes.json");
        let quotes: Vec<QuoteResponse> = serde_json::from_str(json).unwrap();
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, DEFAULT_CHAINFLIP_FEE_BPS);

        assert_eq!(egress_amount.to_string(), "145118751424");
        assert_eq!(slippage_bps, 250);
        assert_eq!(eta_in_seconds, 192);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: None,
                fee_bps: DEFAULT_CHAINFLIP_FEE_BPS,
                estimated_price: "14.5118765424".to_string(),
                dca_parameters: None,
            }
        );
    }

    #[test]
    fn test_best_boost_quote() {
        let json = include_str!("./test/chainflip_boost_quotes.json");
        let quotes: Vec<QuoteResponse> = serde_json::from_str(json).unwrap();
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, DEFAULT_CHAINFLIP_FEE_BPS);

        assert_eq!(egress_amount.to_string(), "4080936927013539226");
        assert_eq!(slippage_bps, 100);
        assert_eq!(eta_in_seconds, 744);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: Some(5),
                fee_bps: DEFAULT_CHAINFLIP_FEE_BPS,
                estimated_price: "40.83388759199201533512".to_string(),
                dca_parameters: Some(DcaParameters {
                    number_of_chunks: 3,
                    chunk_interval: 2,
                }),
            }
        );
    }

    #[tokio::test]
    #[cfg(feature = "swap_integration_tests")]
    async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::alien::reqwest_provider::NativeProvider;
        use primitives::swap::SwapStatus;

        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = ChainflipProvider::new(network_provider.clone());

        // Swap ID: 902663
        let tx_hash = "3sbA7vTDa8tmuokNeQxWJBPpxG3A1Vw5rhDxSm63w7hW31bo2nbci8CfLr27JsbhcebLwcJcwqbL8UP5aVCMFLGb";
        let chain = Chain::Solana;

        let result = swap_provider.get_swap_result(chain, tx_hash).await?;

        println!("Chainflip swap result: {:?}", result);
        assert_eq!(result.from_chain, chain);
        assert_eq!(result.from_tx_hash, tx_hash);
        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.to_chain, Some(Chain::Ethereum));
        assert_eq!(
            result.to_tx_hash,
            Some("0xc142acf0170a2efc7756d9c7c2d27474527ffc4fed6b6c535ca407ffed559dc1".to_string())
        );

        Ok(())
    }
}
