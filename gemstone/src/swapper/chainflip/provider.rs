use alloy_primitives::{U256, hex};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::{str::FromStr, sync::Arc};

use super::{
    ChainflipRouteData,
    broker::{BrokerClient, model::*},
    capitalize::capitalize_first_letter,
    client::{ChainflipClient, QuoteRequest, QuoteResponse},
    price::{apply_slippage, price_to_hex_price},
    seed::generate_random_seed,
    tx_builder,
};
use crate::{
    config::swap_config::DEFAULT_CHAINFLIP_FEE_BPS,
    network::AlienProvider,
    swapper::{
        FetchQuoteData, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperProviderData, SwapperProviderType, SwapperQuote, SwapperQuoteData,
        SwapperQuoteRequest, SwapperRoute, SwapperSwapResult,
        approval::check_approval_erc20,
        asset::{ARBITRUM_USDC, ETHEREUM_FLIP, ETHEREUM_USDC, ETHEREUM_USDT, SOLANA_USDC},
        slippage,
    },
};
use primitives::{ChainType, chain::Chain, swap::QuoteAsset};

const DEFAULT_SWAP_ERC20_GAS_LIMIT: u64 = 100_000;

#[derive(Debug)]
pub struct ChainflipProvider {
    provider: SwapperProviderType,
}

impl Default for ChainflipProvider {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::Chainflip),
        }
    }
}

impl ChainflipProvider {
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

    fn get_best_quote(mut quotes: Vec<QuoteResponse>, fee_bps: u32) -> (BigUint, u32, u32, ChainflipRouteData) {
        quotes.sort_by(|a, b| b.egress_amount.cmp(&a.egress_amount));
        let quote = &quotes[0];

        let egress_amount: BigUint;
        let eta_in_seconds: u32;
        let slippage_bps: u32;
        let boost_fee: Option<u32>;
        let estimated_price: String;
        let dca_parameters: Option<DcaParameters>;

        // Use boost quote if available
        if let Some(boost_quote) = &quote.boost_quote {
            egress_amount = boost_quote.egress_amount.clone();
            slippage_bps = boost_quote.slippage_bps();
            eta_in_seconds = boost_quote.estimated_duration_seconds as u32;
            boost_fee = Some(boost_quote.estimated_boost_fee_bps);
            estimated_price = boost_quote.estimated_price.clone();
            dca_parameters = boost_quote.dca_params.as_ref().map(|dca_params| DcaParameters {
                number_of_chunks: dca_params.number_of_chunks,
                chunk_interval: dca_params.chunk_interval_blocks,
            });
        } else {
            egress_amount = quote.egress_amount.clone();
            slippage_bps = quote.slippage_bps();
            eta_in_seconds = quote.estimated_duration_seconds as u32;
            boost_fee = None;
            estimated_price = quote.estimated_price.clone();
            dca_parameters = quote.dca_params.as_ref().map(|dca_params| DcaParameters {
                number_of_chunks: dca_params.number_of_chunks,
                chunk_interval: dca_params.chunk_interval_blocks,
            });
        }

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
}

#[async_trait::async_trait]
impl Swapper for ChainflipProvider {
    fn provider(&self) -> &SwapperProviderType {
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

    async fn fetch_quote(&self, request: &SwapperQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapperQuote, SwapperError> {
        // Disable swap from BTC until Chainflip scan shows pending transactions
        if request.from_asset.chain().chain_type() == ChainType::Bitcoin {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let src_asset = Self::map_asset_id(&request.from_asset);
        let dest_asset = Self::map_asset_id(&request.to_asset);
        let chainflip_client = ChainflipClient::new(provider.clone());

        let fee_bps = DEFAULT_CHAINFLIP_FEE_BPS;

        let quote_request = QuoteRequest {
            amount: request.value.clone(),
            src_chain: src_asset.chain.clone(),
            src_asset: src_asset.asset.clone(),
            dest_chain: dest_asset.chain,
            dest_asset: dest_asset.asset,
            is_vault_swap: true,
            dca_enabled: true,
            broker_commission_bps: Some(fee_bps),
        };

        let quote_req = chainflip_client.get_quote(&quote_request);
        let quotes = quote_req.await?;

        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = Self::get_best_quote(quotes, fee_bps);

        let quote = SwapperQuote {
            from_value: request.value.clone(),
            to_value: egress_amount.to_string(),
            data: SwapperProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![SwapperRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data).unwrap(),
                    gas_limit: None,
                }],
            },
            eta_in_seconds: Some(eta_in_seconds),
            request: request.clone(),
        };
        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapperQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        let broker_client = BrokerClient::new(provider.clone());
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
                    retry_duration: 150, // blocks
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
                retry_duration: 6, // blocks
            })
        } else if from_asset.chain.chain_type() == ChainType::Solana {
            VaultSwapExtras::Solana(VaultSwapSolanaExtras {
                from: quote.request.wallet_address.clone(),
                seed: hex::encode_prefixed(generate_random_seed(32)),
                chain,
                input_amount: input_amount.to_u64().unwrap(),
                refund_parameters: RefundParameters {
                    retry_duration: 10, // blocks
                    refund_address: quote.request.wallet_address.clone(),
                    min_price,
                },
            })
        } else {
            VaultSwapExtras::None
        };

        let response = broker_client
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
                let value: String = if from_asset.is_native() {
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
                        provider.clone(),
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

                let swap_quote_data = SwapperQuoteData {
                    to: response.to,
                    value,
                    data: response.calldata,
                    approval,
                    gas_limit,
                };

                Ok(swap_quote_data)
            }
            VaultSwapResponse::Bitcoin(response) => {
                let swap_quote_data = SwapperQuoteData {
                    to: response.deposit_address,
                    value: quote.request.value.clone(),
                    data: response.nulldata_payload,
                    approval: None,
                    gas_limit: None,
                };

                Ok(swap_quote_data)
            }
            VaultSwapResponse::Solana(response) => {
                let data = tx_builder::build_solana_tx(&quote.request.wallet_address, &response, provider.clone())
                    .await
                    .map_err(SwapperError::TransactionError)?;
                let swap_quote_data = SwapperQuoteData {
                    to: response.program_id,
                    value: "".into(),
                    data,
                    approval: None,
                    gas_limit: None,
                };

                Ok(swap_quote_data)
            }
        }
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<SwapperSwapResult, SwapperError> {
        let chainflip_client = ChainflipClient::new(provider.clone());
        let status = chainflip_client.get_tx_status(transaction_hash).await?;

        let swap_status = status.swap_status();
        let to_tx_hash = status.swap_egress.as_ref().and_then(|x| x.tx_ref.clone());

        Ok(SwapperSwapResult {
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
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = ChainflipProvider::get_best_quote(quotes, DEFAULT_CHAINFLIP_FEE_BPS);

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
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = ChainflipProvider::get_best_quote(quotes, DEFAULT_CHAINFLIP_FEE_BPS);

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
}
