use rand::Rng;
use std::{str::FromStr, sync::Arc};

use alloy_core::{hex::ToHexExt, primitives::Address};
use alloy_primitives::{keccak256, U256};
use async_trait::async_trait;
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::EthereumRpc,
    mayan::swift::deployment::{get_swift_deployment_by_chain, get_swift_deployment_chains},
};
use num_bigint::RandomBits;
use primitives::{Asset, AssetId, Chain};

use crate::{
    network::{jsonrpc_call, AlienProvider},
    swapper::{
        ApprovalType, FetchQuoteData, GemSwapProvider, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};

use super::{
    fee_manager::{CalcProtocolBpsParams, FeeManager},
    forwarder::MayanForwarder,
    mayan_relayer::{self, MayanRelayer, Quote, QuoteOptions, QuoteParams, QuoteType, MAYAN_FORWARDER_CONTRACT},
    mayan_swift::{KeyStruct, MayanSwift, MayanSwiftError, OrderParams, PermitParams},
};

#[derive(Debug)]
pub struct MayanSwiftProvider {}

impl From<MayanSwiftError> for SwapperError {
    fn from(err: MayanSwiftError) -> Self {
        SwapperError::NetworkError { msg: err.to_string() }
    }
}

impl MayanSwiftProvider {
    pub fn new() -> Self {
        Self {}
    }

    fn get_address_by_chain(chain: Chain) -> Option<String> {
        get_swift_deployment_by_chain(chain).map(|x| x.address)
    }

    async fn check_approval(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<ApprovalType, SwapperError> {
        if request.from_asset.is_native() {
            return Ok(ApprovalType::None);
        }

        let token_id = request.from_asset.token_id.as_ref().ok_or(SwapperError::NotSupportedAsset)?;

        let deployment = get_swift_deployment_by_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;

        let swift_contract = MayanSwift::new(deployment.address, provider.clone(), request.from_asset.chain);

        let amount = &request.value;
        swift_contract
            .check_token_approval(&request.wallet_address, token_id, amount)
            .await
            .map_err(|e| SwapperError::ABIError { msg: e.to_string() })
    }

    fn add_referrer(&self, request: &SwapQuoteRequest, order_params: &mut OrderParams) {
        // TODO: implement if needed
    }

    fn build_swift_order_params(&self, request: &SwapQuoteRequest, quote: &Quote) -> Result<OrderParams, SwapperError> {
        let deadline = quote.deadline64.parse::<u64>().map_err(|_| SwapperError::InvalidRoute)?;

        let trader_address = self.address_to_bytes32(&request.wallet_address)?;
        let destination_address = self.address_to_bytes32(&request.destination_address)?;

        // Handle the output token address
        let token_out = if let to_token_contract = &quote.to_token.contract {
            self.address_to_bytes32(to_token_contract)?
        } else {
            return Err(SwapperError::InvalidAddress {
                address: "Missing to_token contract address".to_string(),
            });
        };

        // Calculate the minimum amount out in smallest unit
        let min_amount_out = self.convert_amount_to_wei(quote.min_amount_out, &request.to_asset)?;

        // Calculate the gas drop in smallest unit
        let gas_drop = self.convert_amount_to_wei(quote.gas_drop, &request.to_asset)?;

        let random_bytes = Self::generate_random_bytes32(); // TODO

        // Handle referrer address
        // let referrer_address = if let Some(referrer) = &request.options {
        //     self.address_to_bytes32(referrer)?
        // } else {
        //     [0u8; 32]
        // };

        // Create the order params
        //
        let params = OrderParams {
            trader: trader_address,
            token_out,
            min_amount_out: min_amount_out.parse().map_err(|_| SwapperError::InvalidAmount)?,
            gas_drop: gas_drop.parse().map_err(|_| SwapperError::InvalidAmount)?,
            cancel_fee: quote.cancel_relayer_fee64.parse::<u64>().map_err(|_| SwapperError::InvalidAmount)?,
            refund_fee: quote.refund_relayer_fee64.parse::<u64>().map_err(|_| SwapperError::InvalidAmount)?,
            deadline: quote.deadline64.parse().map_err(|_| SwapperError::InvalidRoute)?,
            dest_addr: destination_address,
            dest_chain_id: request.to_asset.chain.network_id().parse().map_err(|_| SwapperError::InvalidAmount)?,
            referrer_addr: [0u8; 32], // Add referrer logic if applicable
            referrer_bps: 0u8,
            auction_mode: quote.swift_auction_mode.unwrap_or(0),
            random: random_bytes,
        };

        Ok(params)
    }

    fn generate_random_bytes32() -> [u8; 32] {
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 32];
        rng.fill(&mut random_bytes);
        random_bytes
    }

    fn address_to_bytes32(&self, address: &str) -> Result<[u8; 32], SwapperError> {
        let addr = EthereumAddress::from_str(address).map_err(|_| SwapperError::InvalidAddress { address: address.to_string() })?;
        let mut bytes32 = [0u8; 32];
        bytes32[12..].copy_from_slice(&addr.bytes);
        Ok(bytes32)
    }

    async fn fetch_quote_from_request(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<Quote, SwapperError> {
        let mayan_relayer = MayanRelayer::default_relayer(provider.clone());
        let quote_params = QuoteParams {
            amount: request.value.parse().map_err(|_| SwapperError::InvalidAmount)?,
            from_token: request.from_asset.token_id.clone().unwrap_or(EthereumAddress::zero().to_checksum()),
            from_chain: request.from_asset.chain.clone(),
            to_token: request.to_asset.token_id.clone().unwrap_or(EthereumAddress::zero().to_checksum()),
            to_chain: request.to_asset.chain.clone(),
            slippage_bps: Some(100),
            gas_drop: None,
            referrer: None,
            referrer_bps: None,
        };

        let quote_options = QuoteOptions {
            swift: true,
            mctp: false,
            gasless: false,
            only_direct: false,
        };

        let quote = mayan_relayer
            .get_quote(quote_params, Some(quote_options))
            .await
            .map_err(|e| SwapperError::ComputeQuoteError {
                msg: format!("Mayan relayer quote error: {:?}", e),
            })?;

        // TODO: adjust to find most effective quote
        let most_effective_qoute = quote.into_iter().filter(|x| x.r#type == QuoteType::SWIFT.to_string()).last();

        most_effective_qoute.ok_or(SwapperError::ComputeQuoteError {
            msg: "Quote is not available".to_string(),
        })
    }

    fn convert_amount_to_wei(&self, amount: f64, asset_id: &AssetId) -> Result<String, SwapperError> {
        // Retrieve asset information based on the provided AssetId
        let asset = Asset::from_chain(asset_id.chain);

        // Get the decimals for the asset
        let decimals = asset.decimals;

        // Calculate the scaling factor (10^decimals)
        let scaling_factor = 10f64.powi(decimals as i32);

        // Convert the amount to Wei (or the smallest unit)
        let amount_in_wei = (amount * scaling_factor).round(); // `round` ensures correct conversion

        // Ensure the amount is within a valid range for integers
        if amount_in_wei < 0.0 || amount_in_wei > (u64::MAX as f64) {
            return Err(SwapperError::InvalidAmount);
        }

        // Convert the result to a string for return
        Ok(format!("{:.0}", amount_in_wei))
    }
}

#[async_trait]
impl GemSwapProvider for MayanSwiftProvider {
    fn provider(&self) -> SwapProvider {
        SwapProvider::MayanSwift
    }

    fn supported_chains(&self) -> Vec<primitives::Chain> {
        get_swift_deployment_chains()
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;
        let mayan_quote = self.fetch_quote_from_request(&request, provider.clone()).await?;
        let swift_address = if let Some(address) = &mayan_quote.swift_mayan_contract {
            address.clone()
        } else {
            return Err(SwapperError::ComputeQuoteError {
                msg: "No swift_mayan_contract in quote".to_string(),
            });
        };
        let swift_contract = MayanSwift::new(swift_address.clone(), provider.clone(), request.from_asset.chain);
        let swift_order_params = self.build_swift_order_params(&quote.request, &mayan_quote)?;
        let forwarder = MayanForwarder::new(MAYAN_FORWARDER_CONTRACT.to_string(), provider.clone(), request.from_asset.chain);

        let swift_call_data = if request.from_asset.is_native() {
            swift_contract
                .encode_create_order_with_eth(swift_order_params, quote.from_value.parse().map_err(|_| SwapperError::InvalidAmount)?)
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        } else {
            swift_contract
                .encode_create_order_with_token(
                    request.from_asset.token_id.as_ref().ok_or(SwapperError::InvalidAddress {
                        address: request.from_asset.to_string(),
                    })?,
                    quote.from_value.parse().map_err(|_| SwapperError::InvalidAmount)?,
                    swift_order_params,
                )
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        };

        let mut value = quote.from_value.clone();

        let forwarder_call_data = if request.from_asset.is_native() {
            forwarder
                .encode_forward_eth_call(swift_address.as_str(), swift_call_data.clone())
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        } else {
            todo!()
        };

        Ok(SwapQuoteData {
            to: MAYAN_FORWARDER_CONTRACT.to_string(),
            value: value.clone(),
            data: forwarder_call_data.encode_hex(),
        })
    }

    async fn get_transaction_status(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        todo!();
        // let receipt_call = EthereumRpc::GetTransactionReceipt(transaction_hash.to_string());
        //
        // let response = jsonrpc_call(&receipt_call, provider, &chain)
        //     .await
        //     .map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        //
        // let result: serde_json::Value = response.extract_result().map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        //
        // if let Some(status_hex) = result.get("status").and_then(|s| s.as_str()) {
        //     let status = U256::from_str_radix(status_hex.trim_start_matches("0x"), 16).unwrap_or_else(|_| U256::zero());
        //     Ok(!status.is_zero())
        // } else {
        //     Ok(false)
        // }
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // Validate chain support
        if !self.supported_chains().contains(&request.from_asset.chain) {
            return Err(SwapperError::NotSupportedChain);
        }

        let quote = self.fetch_quote_from_request(request, provider.clone()).await?;

        if quote.r#type != QuoteType::SWIFT.to_string() {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Quote type is not SWIFT".to_string(),
            });
        }

        // Create route information
        let route = SwapRoute {
            route_type: "swift-order".to_string(),
            input: request
                .from_asset
                .token_id
                .clone()
                .unwrap_or_else(|| request.from_asset.chain.as_ref().to_string()),
            output: request.to_asset.token_id.clone().unwrap_or_else(|| request.to_asset.chain.as_ref().to_string()),
            fee_tier: "0".to_string(),
            gas_estimate: None,
        };

        let approval = self.check_approval(request, provider.clone()).await?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: self
                .convert_amount_to_wei(quote.min_amount_out, &request.to_asset)
                .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?,
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![route],
            },
            approval,
            request: request.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use alloy_core::sol_types::SolValue;
    use alloy_primitives::U256;
    use primitives::AssetId;

    use crate::{
        network::{AlienError, AlienTarget, Data},
        swapper::GemSwapMode,
    };

    use super::*;

    #[test]
    fn test_eth_value_conversion() {
        let decimal_str = "1000000000000000000"; // 1 ETH
        let value = U256::from_str(decimal_str).unwrap();

        let hex_value = format!("0x{}", value.to_string().encode_hex());

        assert_eq!(hex_value, "0xde0b6b3a7640000");
    }

    #[test]
    fn test_supported_chains() {
        let provider = MayanSwiftProvider::new();
        let chains = provider.supported_chains();

        assert!(chains.contains(&Chain::Solana));
        assert!(chains.contains(&Chain::Ethereum));
        assert!(chains.contains(&Chain::SmartChain));
        assert!(chains.contains(&Chain::Polygon));
        assert!(chains.contains(&Chain::Arbitrum));
        assert!(chains.contains(&Chain::Optimism));
        assert!(chains.contains(&Chain::Base));
    }
}
