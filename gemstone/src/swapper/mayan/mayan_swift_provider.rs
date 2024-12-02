use rand::Rng;
use std::{str::FromStr, sync::Arc};

use alloy_core::{hex::ToHexExt, primitives::Address};
use alloy_primitives::{keccak256, U256};
use async_trait::async_trait;
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::EthereumRpc,
    mayan::swift::deployment::{get_swift_deployment_by_chain, get_swift_deployment_chains, get_swift_providers},
};
use primitives::{Asset, AssetId, Chain};

use crate::{
    config::swap_config::SwapReferralFee,
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

const MAYAN_ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

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

    fn get_chain_by_wormhole_id(&self, wormhole_id: u64) -> Option<Chain> {
        get_swift_providers()
            .into_iter()
            .find(|(chain, deployment)| deployment.wormhole_id == wormhole_id)
            .map(|(chain, _)| chain)
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

    fn to_native_wormhole_address(&self, address: &str, w_chain_id: u64) -> Result<[u8; 32], SwapperError> {
        let chain = self.get_chain_by_wormhole_id(w_chain_id).ok_or(SwapperError::InvalidRoute)?;

        if chain == Chain::Solana {
            todo!()
        } else {
            Ok(self.address_to_bytes32(address)?)
        }
    }

    fn build_swift_order_params(&self, request: &SwapQuoteRequest, quote: &Quote) -> Result<OrderParams, SwapperError> {
        let dest_chain_id = quote.to_token.w_chain_id.unwrap();
        let from_chain_id = quote.from_token.w_chain_id.unwrap();

        let deadline = quote.deadline64.parse::<u64>().map_err(|_| {
            eprintln!("Failed to parse deadline: {}", quote.deadline64);
            SwapperError::InvalidRoute
        })?;

        let trader_address = self.address_to_bytes32(&request.wallet_address).map_err(|e| {
            eprintln!("Failed to parse wallet_address: {}", request.wallet_address);
            e
        })?;

        let destination_address = self.address_to_bytes32(&request.destination_address).map_err(|e| {
            eprintln!("Failed to parse destination_address: {}", request.destination_address);
            e
        })?;

        let token_out = if let to_token_contract = &quote.to_token.contract {
            self.address_to_bytes32(to_token_contract).map_err(|e| {
                eprintln!("Failed to parse to_token.contract: {}", to_token_contract);
                e
            })?
        } else {
            return Err(SwapperError::InvalidAddress {
                address: "Missing to_token contract address".to_string(),
            });
        };

        let min_amount_out = self
            .get_amount_of_fractional_amount(quote.min_amount_out, quote.to_token.decimals)
            .map_err(|e| {
                eprintln!("Failed to convert min_amount_out: {}", quote.min_amount_out);
                e
            })?;

        let gas_drop = self.convert_amount_to_wei(quote.gas_drop, &request.to_asset).map_err(|e| {
            eprintln!("Failed to convert gas_drop: {}", quote.gas_drop);
            e
        })?;

        let random_bytes = Self::generate_random_bytes32();

        let referrer = self.get_referrer(request)?;

        let params = OrderParams {
            trader: trader_address,
            token_out,
            min_amount_out: min_amount_out.parse().map_err(|_| SwapperError::InvalidAmount)?,
            gas_drop: gas_drop.parse().map_err(|_| SwapperError::InvalidAmount)?,
            cancel_fee: quote.cancel_relayer_fee64.parse::<u64>().map_err(|_| SwapperError::InvalidAmount)?,
            refund_fee: quote.refund_relayer_fee64.parse::<u64>().map_err(|_| SwapperError::InvalidAmount)?,
            deadline,
            dest_addr: destination_address,
            dest_chain_id: dest_chain_id.to_string().parse().map_err(|_| SwapperError::InvalidAmount)?,
            referrer_addr: self
                .to_native_wormhole_address(referrer.address.as_str(), from_chain_id)
                .map_err(|e| SwapperError::InvalidAddress {
                    address: referrer.address.clone(),
                })?,
            referrer_bps: referrer.bps.try_into().map_err(|_| SwapperError::InvalidAmount)?,
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

    fn get_referrer(&self, request: &SwapQuoteRequest) -> Result<SwapReferralFee, SwapperError> {
        if let Some(options) = &request.options {
            if let Some(referrer) = &options.fee {
                let evm_fee = &referrer.evm;
                let solana_fee = &referrer.solana;

                if request.from_asset.chain == Chain::Solana {
                    return Ok(solana_fee.clone());
                }

                return Ok(evm_fee.clone());
            }
        }

        Err(SwapperError::ComputeQuoteError {
            msg: "Missing referrer".to_string(),
        })
    }

    async fn fetch_quote_from_request(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<Quote, SwapperError> {
        let mayan_relayer = MayanRelayer::default_relayer(provider.clone());
        let referrer = self.get_referrer(request)?;
        let quote_params = QuoteParams {
            amount: request.value.parse().map_err(|_| SwapperError::InvalidAmount)?,
            from_token: request.from_asset.token_id.clone().unwrap_or(EthereumAddress::zero().to_checksum()),
            from_chain: request.from_asset.chain.clone(),
            to_token: request.to_asset.token_id.clone().unwrap_or(EthereumAddress::zero().to_checksum()),
            to_chain: request.to_asset.chain.clone(),
            slippage_bps: Some(100),
            gas_drop: None,
            referrer: Some(referrer.address),
            referrer_bps: Some(referrer.bps),
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

    fn get_amount_of_fractional_amount(&self, amount: f64, decimals: u8) -> Result<String, SwapperError> {
        if amount < 0.0 || !amount.is_finite() {
            return Err(SwapperError::InvalidAmount);
        }

        // Determine the cut factor (maximum of 8 or the provided decimals)
        let cut_factor = std::cmp::min(8, decimals as i32);

        // Format the amount to cut_factor + 1 decimal places
        let formatted_amount = format!("{:.precision$}", amount, precision = (cut_factor + 1) as usize);

        // Extract and truncate to cut_factor decimal places
        let truncated_amount = if let Some((int_part, decimal_part)) = formatted_amount.split_once('.') {
            let truncated_decimals = &decimal_part[..std::cmp::min(decimal_part.len(), cut_factor as usize)];
            format!("{}.{}", int_part, truncated_decimals)
        } else {
            formatted_amount
        };

        // Calculate the result scaled by 10^cut_factor
        let scaled_amount = truncated_amount.parse::<f64>().map_err(|_| SwapperError::InvalidAmount)? * 10f64.powi(cut_factor);

        // Validate range
        if scaled_amount < 0.0 || scaled_amount > (u64::MAX as f64) {
            return Err(SwapperError::InvalidAmount);
        }

        // Return the scaled amount as a string
        Ok(format!("{:.0}", scaled_amount))
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
        let forwarder = MayanForwarder::new();

        let swift_call_data = if mayan_quote.swift_input_contract == MAYAN_ZERO_ADDRESS {
            swift_contract
                .encode_create_order_with_eth(swift_order_params, quote.from_value.parse().map_err(|_| SwapperError::InvalidAmount)?)
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        } else {
            swift_contract
                .encode_create_order_with_token(
                    mayan_quote.swift_input_contract.as_str(),
                    quote.from_value.parse().map_err(|_| SwapperError::InvalidAmount)?,
                    swift_order_params,
                )
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        };

        let mut value = quote.from_value.clone();
        let forwarder_call_data = if mayan_quote.from_token.contract == mayan_quote.swift_input_contract {
            if mayan_quote.from_token.contract == MAYAN_ZERO_ADDRESS {
                forwarder
                    .encode_forward_eth_call(swift_address.as_str(), swift_call_data.clone())
                    .await
                    .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            } else {
                todo!();
            }
        } else {
            let evm_swap_router_address = mayan_quote.evm_swap_router_address.clone().ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing evmSwapRouterAddress".to_string(),
            })?;
            let evm_swap_router_calldata = mayan_quote.evm_swap_router_calldata.clone().ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing evmSwapRouterCalldata".to_string(),
            })?;
            let min_middle_amount = mayan_quote.min_middle_amount.clone().ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing minMiddleAmount".to_string(),
            })?;

            let token_in = mayan_quote.from_token.contract.clone();
            let formatted_min_middle_amount = self
                .get_amount_of_fractional_amount(min_middle_amount, mayan_quote.swift_input_decimals)
                .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;
            let effective_amount_in = self
                .get_amount_of_fractional_amount(mayan_quote.effective_amount_in, mayan_quote.from_token.decimals)
                .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

            if (mayan_quote.from_token.contract == MAYAN_ZERO_ADDRESS) {
                forwarder
                    .encode_swap_and_forward_eth_call(
                        U256::from_str(quote.from_value.as_str()).map_err(|_| SwapperError::InvalidAmount)?,
                        &evm_swap_router_address.as_str(),
                        hex::decode(evm_swap_router_calldata.trim_start_matches("0x")).map_err(|_| SwapperError::ABIError {
                            msg: "Failed to decode evm_swap_router_calldata hex string without prefix 0x ".to_string(),
                        })?,
                        &mayan_quote.swift_input_contract.as_str(),
                        U256::from_str(&formatted_min_middle_amount).map_err(|_| SwapperError::InvalidAmount)?,
                        &swift_address.as_str(),
                        swift_call_data,
                    )
                    .await
                    .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            } else {
                todo!();
            }
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
    use serde::{Deserialize, Deserializer, Serialize};

    use crate::{
        network::{AlienError, AlienTarget, Data},
        swapper::GemSwapMode,
    };

    use super::*;

    pub fn generate_mock_quote() -> Quote {
        let json_data = r#"
        {
      "maxUserGasDrop": 0.01,
      "sendTransactionCost": 0,
      "rentCost": "40000000",
      "gasless": false,
      "swiftAuctionMode": 2,
      "swiftMayanContract": "0xC38e4e6A15593f908255214653d3D947CA1c2338",
      "minMiddleAmount": null,
      "swiftInputContract": "0x0000000000000000000000000000000000000000",
      "swiftInputDecimals": 18,
      "slippageBps": 100,
      "effectiveAmountIn": 0.1,
      "expectedAmountOut": 0.09972757016582551,
      "price": 0.9996900030000002,
      "priceImpact": null,
      "minAmountOut": 0.09873029446416727,
      "minReceived": 0.09873029446416727,
      "route": null,
      "swapRelayerFee": null,
      "swapRelayerFee64": null,
      "redeemRelayerFee": null,
      "redeemRelayerFee64": null,
      "solanaRelayerFee": null,
      "solanaRelayerFee64": null,
      "refundRelayerFee": null,
      "refundRelayerFee64": "1056",
      "cancelRelayerFee64": "25",
      "submitRelayerFee64": "0",
      "deadline64": "1732777812",
      "clientRelayerFeeSuccess": null,
      "clientRelayerFeeRefund": 0.038947098479114796,
      "fromToken": {
        "name": "ETH",
        "standard": "native",
        "symbol": "ETH",
        "mint": "",
        "verified": true,
        "contract": "0x0000000000000000000000000000000000000000",
        "wrappedAddress": "0x4200000000000000000000000000000000000006",
        "chainId": 8453,
        "wChainId": 30,
        "decimals": 18,
        "logoURI": "https://statics.mayan.finance/eth.png",
        "coingeckoId": "weth",
        "pythUsdPriceId": "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        "realOriginContractAddress": "0x4200000000000000000000000000000000000006",
        "realOriginChainId": 30,
        "supportsPermit": false,
        "hasAuction": true
      },
      "fromChain": "base",
      "toToken": {
        "name": "ETH",
        "standard": "native",
        "symbol": "ETH",
        "mint": "8M6d63oL7dvMZ1gNbgGe3h8afMSWJEKEhtPTFM2u8h3c",
        "verified": true,
        "contract": "0x0000000000000000000000000000000000000000",
        "wrappedAddress": "0x4200000000000000000000000000000000000006",
        "chainId": 10,
        "wChainId": 24,
        "decimals": 18,
        "logoURI": "https://statics.mayan.finance/eth.png",
        "coingeckoId": "weth",
        "pythUsdPriceId": "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        "realOriginContractAddress": "0x4200000000000000000000000000000000000006",
        "realOriginChainId": 24,
        "supportsPermit": false,
        "hasAuction": true
      },
      "toTokenPrice": 3602.87682508,
      "toChain": "optimism",
      "mintDecimals": null,
      "gasDrop": 0,
      "eta": 1,
      "etaSeconds": 12,
      "clientEta": "12s",
      "bridgeFee": 0,
      "suggestedPriorityFee": 0,
      "type": "SWIFT",
      "priceStat": {
        "ratio": 0.9996907516582549,
        "status": "GOOD"
      },
      "referrerBps": 0,
      "protocolBps": 3,
      "onlyBridging": false,
      "sourceSwapExpense": 0,
      "relayer": "7dm9am6Qx7cH64RB99Mzf7ZsLbEfmXM7ihXXCvMiT2X1",
      "meta": {
        "advertisedDescription": "Cheapest and Fastest",
        "advertisedTitle": "Best",
        "icon": "https://cdn.mayan.finance/fast_icon.png",
        "switchText": "Switch to the best route",
        "title": "Best"
      }
    }
        "#;

        let quote: Quote = serde_json::from_str(json_data).expect("Failed to deserialize Quote");
        quote
    }

    /// Generates a `SwapQuoteRequest` object using values directly.
    pub fn generate_mock_request() -> SwapQuoteRequest {
        SwapQuoteRequest {
            wallet_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            destination_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            from_asset: AssetId {
                chain: Chain::Base,
                token_id: None,
            },
            to_asset: AssetId {
                chain: Chain::Optimism,
                token_id: None,
            },
            value: "1230000000000000000".to_string(),
            mode: GemSwapMode::ExactIn,
            options: None, // From JSON field "effectiveAmountIn"
        }
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

    #[test]
    fn test_address_to_bytes32_valid() {
        let provider = MayanSwiftProvider::new();
        let address = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";
        let bytes32 = provider.address_to_bytes32(address).unwrap();
        let expected_bytes32 = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 85, 198, 171, 218, 94, 42, 82, 65, 170, 8, 72, 107, 213, 12, 247, 212, 117, 207, 36,
        ];
        assert_eq!(bytes32, expected_bytes32);
    }

    #[test]
    fn test_address_to_bytes32_invalid() {
        let provider = MayanSwiftProvider::new();
        let invalid_address = "invalid_address";
        let result = provider.address_to_bytes32(invalid_address);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_amount_to_wei_valid() {
        let provider = MayanSwiftProvider::new();
        let asset = Asset::from_chain(Chain::Ethereum);
        let amount = 1.23; // 1.23 ETH
        let result = provider.convert_amount_to_wei(amount, &asset.id).unwrap();
        assert_eq!(result, "1230000000000000000"); // 1.23 ETH in Wei
    }

    #[test]
    fn test_convert_amount_to_wei_invalid() {
        let provider = MayanSwiftProvider::new();
        let asset = Asset::from_chain(Chain::Ethereum);
        let amount = -1.0; // Negative amount
        let result = provider.convert_amount_to_wei(amount, &asset.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_swift_order_params_valid() {
        let provider = MayanSwiftProvider::new();
        let request = generate_mock_request();
        let quote = generate_mock_quote();

        let result = provider.build_swift_order_params(&request, &quote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_amount_of_fractional_amount_valid() {
        // Test with valid inputs and expected results
        let test_cases = vec![
            (0.000075203, 18, 7520),    // Regular case with precision truncation
            (1.23456789, 8, 123456789), // Decimals less than 8
            (0.1, 6, 100000),           // Simple rounding
            (0.12345678, 8, 12345678),  // Exact decimals
        ];

        for (amount, decimals, expected) in test_cases {
            let provider = MayanSwiftProvider::new();
            let result = provider.get_amount_of_fractional_amount(amount, decimals);
            assert!(result.is_ok(), "Failed for amount: {}", amount);
            assert_eq!(result.unwrap(), expected.to_string());
        }
    }

    #[test]
    fn test_decode_evm_calldata_valid() {
        let calldata = "0x07ed2379000000000000000000000000e37e799d5077682fa0a244d46e5649f71457bd09000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913000000000000000000000000e37e799d5077682fa0a244d46e5649f71457bd090000000000000000000000000654874eb7f59c6f5b39931fc45dc45337c967c3000000000000000000000000000000000000000000000000016345785d8a00000000000000000000000000000000000000000000000000000000000013db374b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000004720000000000000000000000000000000000000000000004540004260003dc416003c01acae3d0173a93d819efdc832c7c4f153b06016452bbbe2900000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000e37e799d5077682fa0a244d46e5649f71457bd090000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e37e799d5077682fa0a244d46e5649f71457bd0900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006749f4bcdef66c6c178087fd931514e99b04479e4d3d956c00020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913000000000000000000000000000000000000000000000000016345785d8a000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000006280fa0775121000000000000000000000000000000000000000000000000000000006749f4bc00000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000654874eb7f59c6f5b39931fc45dc45337c967c300000000000000000000000000000000000000000000000000005af3107a400000000000000000000000fc892ee4a97800000000000000000166d2f7025080000000000000000003f0a5341099c401550000000000002eb7b4329f0233e9100000000000000000000000000000000000000000000004ee7259d6914ae6c461bc00000000000000004563918244f400000000000000000000006a94d74f4300000000000000000000000000006749f462000000000000000000003b1dfde910000000000000000000000000000000000000000000000000000000000000000041923845af1a0f8e52538eb26d2d7a9d0e9fc833f801c72890cc3aec84fbc1b930696a280a0414c1cab407169bb90cf3c122f85383e9bf614e490c181bcca205b41b0000000000000000000000000000000000000000000000000000000000000000a0f2fa6b66833589fcd6edb6e08f4c7c32d4f71b54bda029130000000000000000000000000000000000000000000000000000000015775e5f000000000000000000000000004c421380a06c4eca27833589fcd6edb6e08f4c7c32d4f71b54bda02913111111125421ca6dc452d289314280a0f8842a6500000000000000000000000000001ea79a4f";
        let without_prefix = calldata.trim_start_matches("0x");
        let decoded_calldata = hex::decode(without_prefix).unwrap();
        let encoded_calldata = hex::encode(&decoded_calldata);
        let bytes = calldata.as_bytes();

        assert_eq!(encoded_calldata, calldata);
        assert_eq!(bytes, decoded_calldata);
    }
}
