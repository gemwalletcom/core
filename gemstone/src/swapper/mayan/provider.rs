use rand::Rng;
use std::{str::FromStr, sync::Arc};

use alloy_core::{
    hex::{decode as HexDecode, ToHexExt},
    primitives::{utils::parse_units, U256},
    sol_types::SolCall,
};

use async_trait::async_trait;
use gem_evm::{
    address::EthereumAddress,
    erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    mayan::deployment::get_swift_providers,
};
use primitives::{Asset, AssetId, Chain};

use crate::{
    config::swap_config::SwapReferralFee,
    network::{jsonrpc_call, AlienProvider},
    swapper::{
        approval::{check_approval, CheckApprovalType},
        ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData,
        SwapQuoteRequest, SwapRoute, SwapperError,
    },
};

use super::{
    forwarder::MayanForwarder,
    relayer::{MayanRelayer, Quote, QuoteOptions, QuoteParams, QuoteType},
    swift::{MayanSwift, MayanSwiftError, OrderParams},
    MAYAN_FORWARDER_CONTRACT, MAYAN_ZERO_ADDRESS,
};

#[derive(Debug)]
pub struct MayanSwiftProvider {
    provider: SwapProviderType,
}

impl Default for MayanSwiftProvider {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::MayanSwift),
        }
    }
}

impl From<MayanSwiftError> for SwapperError {
    fn from(err: MayanSwiftError) -> Self {
        SwapperError::NetworkError { msg: err.to_string() }
    }
}

impl MayanSwiftProvider {
    fn get_chain_by_wormhole_id(&self, wormhole_id: u64) -> Option<Chain> {
        get_swift_providers()
            .into_iter()
            .find(|(_, deployment)| deployment.wormhole_id.clone() as u64 == wormhole_id)
            .map(|(chain, _)| chain)
    }

    async fn check_approval(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<ApprovalType, SwapperError> {
        if request.from_asset.is_native() {
            return Ok(ApprovalType::None);
        }

        check_approval(
            CheckApprovalType::ERC20 {
                owner: request.wallet_address.clone(),
                token: request.from_asset.token_id.clone().unwrap_or_default(),
                spender: MAYAN_FORWARDER_CONTRACT.to_string(),
                amount: U256::from_str(request.value.as_str()).map_err(|_| SwapperError::InvalidAmount)?,
            },
            provider,
            &request.from_asset.chain,
        )
        .await
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

        let deadline = quote.deadline64.parse::<u64>().map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Failed to parse deadline".to_string(),
        })?;

        let trader_address = self.address_to_bytes32(&request.wallet_address)?;
        let destination_address = self.address_to_bytes32(&request.destination_address)?;
        let token_out = self.address_to_bytes32(&quote.to_token.contract)?;

        let min_amount_out = self.get_amount_of_fractional_amount(quote.min_amount_out, quote.to_token.decimals)?;
        // TODO: check if we need to use to token or from token decimals
        let gas_drop = self.convert_amount_to_wei(quote.gas_drop, quote.to_token.decimals).inspect_err(|e| {
            eprintln!("Failed to convert gas_drop: {}, {}", quote.gas_drop, e);
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
            referrer_addr: referrer.clone().map_or([0u8; 32], |x| {
                self.to_native_wormhole_address(x.address.as_str(), from_chain_id)
                    .map_err(|_| SwapperError::ComputeQuoteError {
                        msg: "Unable to get referrer wormhole address".to_string(),
                    })
                    .unwrap()
            }),
            referrer_bps: referrer.map_or(0u8, |x| x.bps.try_into().map_err(|_| SwapperError::InvalidAmount).unwrap()),
            auction_mode: quote.swift_auction_mode.unwrap_or(0),
            random: random_bytes,
        };

        Ok(params)
    }

    fn generate_random_bytes32() -> [u8; 32] {
        let mut rng = rand::rng();
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

    fn get_referrer(&self, request: &SwapQuoteRequest) -> Result<Option<SwapReferralFee>, SwapperError> {
        if let Some(referrer) = &request.options.fee {
            let evm_fee = &referrer.evm;
            let solana_fee = &referrer.solana;

            if request.from_asset.chain == Chain::Solana {
                return Ok(Some(solana_fee.clone()));
            }

            return Ok(Some(evm_fee.clone()));
        }

        Ok(None)
    }

    async fn fetch_quote_from_request(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<Quote, SwapperError> {
        let asset_decimals = self.get_asset_decimals(request.from_asset.clone(), provider.clone()).await?;
        let mayan_relayer = MayanRelayer::default_relayer(provider.clone());
        let referrer = self.get_referrer(request)?;
        let quote_params = QuoteParams {
            amount: self.convert_to_decimals(
                request.value.parse().map_err(|_| SwapperError::ComputeQuoteError {
                    msg: "Failed to convert request value to number".to_string(),
                })?,
                asset_decimals,
            ),
            from_token: request.from_asset.token_id.clone().unwrap_or(EthereumAddress::zero().to_checksum()),
            from_chain: request.from_asset.chain,
            to_token: request.to_asset.token_id.clone().unwrap_or(EthereumAddress::zero().to_checksum()),
            to_chain: request.to_asset.chain,
            slippage_bps: Some(100),
            gas_drop: None,
            referrer: referrer.clone().map(|x| x.address),
            referrer_bps: referrer.map(|x| x.bps),
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
        let most_effective_quote = quote.into_iter().filter(|x| x.r#type == QuoteType::Swift.to_string()).last();

        most_effective_quote.ok_or(SwapperError::ComputeQuoteError {
            msg: "Quote is not available".to_string(),
        })
    }

    fn convert_amount_to_wei(&self, amount: f64, decimals: u8) -> Result<String, SwapperError> {
        if amount < 0.0 {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Cannot convert negative amount".to_string(),
            });
        }

        let parsed = parse_units(amount.to_string().as_str(), decimals).map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Invalid conversion amount to decimals".to_string(),
        })?;

        Ok(parsed.to_string())
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

    async fn get_asset_decimals(&self, asset_id: AssetId, provider: Arc<dyn AlienProvider>) -> Result<u32, SwapperError> {
        let asset = Asset::from_chain(asset_id.chain);

        if asset_id.is_native() {
            return Ok(asset.decimals as u32);
        }
        let address = asset_id.token_id.clone().unwrap();
        let decimals_data = IERC20::decimalsCall {}.abi_encode();
        let decimals_call = EthereumRpc::Call(TransactionObject::new_call(&address, decimals_data), BlockParameter::Latest);

        let response = jsonrpc_call(&decimals_call, provider.clone(), &asset_id.chain)
            .await
            .map_err(|err| SwapperError::ComputeQuoteError {
                msg: format!("Failed to get ERC20 decimals: {}", err),
            })?;
        let result: String = response.take().map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Failed to get ERC20 decimals".to_string(),
        })?;
        let decoded = HexDecode(result).map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Failed to decode decimals return".to_string(),
        })?;
        let decimals_return = IERC20::decimalsCall::abi_decode_returns(&decoded, false).map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Failed to decode decimals return".to_string(),
        })?;

        Ok(decimals_return._0.into())
    }

    fn convert_to_decimals(&self, wei_amount: u128, decimals: u32) -> f64 {
        let divisor = 10_u64.pow(decimals);
        wei_amount as f64 / divisor as f64
    }
}

#[async_trait]
impl GemSwapProvider for MayanSwiftProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            SwapChainAsset::All(Chain::Solana),
            SwapChainAsset::All(Chain::Polygon),
            SwapChainAsset::All(Chain::SmartChain),
            SwapChainAsset::All(Chain::Ethereum),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let quote = self.fetch_quote_from_request(request, provider.clone()).await?;

        if quote.r#type != QuoteType::Swift.to_string() {
            return Err(SwapperError::ComputeQuoteError {
                msg: "Quote type is not SWIFT".to_string(),
            });
        }

        // Create route information
        let route = SwapRoute {
            route_data: "swift-order".to_string(),
            input: request.from_asset.clone(),
            output: request.to_asset.clone(),
            gas_limit: None,
        };

        let to_value = self
            .convert_amount_to_wei(quote.min_amount_out, quote.to_token.decimals)
            .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: to_value.clone(),
            to_min_value: to_value, // FIXME
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps: quote.slippage_bps,
                routes: vec![route],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;
        let mayan_quote = self.fetch_quote_from_request(request, provider.clone()).await?;
        let swift_address = mayan_quote.swift_mayan_contract.clone().ok_or(SwapperError::ComputeQuoteError {
            msg: "No swift_mayan_contract in quote".to_string(),
        })?;
        let swift_contract = MayanSwift::default();
        let swift_order_params = self.build_swift_order_params(&quote.request, &mayan_quote)?;
        let forwarder = MayanForwarder::default();

        let swift_call_data = if mayan_quote.swift_input_contract == MAYAN_ZERO_ADDRESS {
            swift_contract
                .encode_create_order_with_eth(swift_order_params)
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
        let effective_amount_in = self
            .get_amount_of_fractional_amount(mayan_quote.effective_amount_in, mayan_quote.from_token.decimals)
            .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        let forwarder_call_data = if mayan_quote.from_token.contract == mayan_quote.swift_input_contract {
            if mayan_quote.from_token.contract == MAYAN_ZERO_ADDRESS {
                forwarder
                    .encode_forward_eth_call(swift_address.as_str(), swift_call_data.clone())
                    .await
                    .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            } else {
                value = "0".to_string();
                forwarder
                    .encode_forward_erc20_call(
                        mayan_quote.swift_input_contract.as_str(),
                        U256::from_str(effective_amount_in.as_str()).unwrap(),
                        None,
                        swift_address.as_str(),
                        swift_call_data.clone(),
                    )
                    .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            }
        } else {
            let evm_swap_router_address = mayan_quote.evm_swap_router_address.clone().ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing evmSwapRouterAddress".to_string(),
            })?;
            let evm_swap_router_calldata = mayan_quote.evm_swap_router_calldata.clone().ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing evmSwapRouterCalldata".to_string(),
            })?;
            let min_middle_amount = mayan_quote.min_middle_amount.ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing minMiddleAmount".to_string(),
            })?;

            let token_in = mayan_quote.from_token.contract.clone();
            let formatted_min_middle_amount = self
                .get_amount_of_fractional_amount(min_middle_amount, mayan_quote.swift_input_decimals)
                .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

            let amount_in = U256::from_str(quote.from_value.as_str()).map_err(|_| SwapperError::InvalidAmount)?;
            let swap_data = hex::decode(evm_swap_router_calldata.trim_start_matches("0x")).map_err(|_| SwapperError::ABIError {
                msg: "Failed to decode evm_swap_router_calldata hex string without prefix 0x ".to_string(),
            })?;
            let min_middle_amount = U256::from_str(&formatted_min_middle_amount).map_err(|_| SwapperError::InvalidAmount)?;

            if mayan_quote.from_token.contract == MAYAN_ZERO_ADDRESS {
                forwarder
                    .encode_swap_and_forward_eth_call(
                        amount_in,
                        evm_swap_router_address.as_str(),
                        swap_data,
                        mayan_quote.swift_input_contract.as_str(),
                        min_middle_amount,
                        swift_address.as_str(),
                        swift_call_data,
                    )
                    .await
                    .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            } else {
                value = "0".to_string();
                forwarder
                    .encode_swap_and_forward_erc20_call(
                        token_in.as_str(),
                        amount_in,
                        None,
                        evm_swap_router_address.as_str(),
                        swap_data,
                        mayan_quote.swift_input_contract.as_str(),
                        min_middle_amount,
                        swift_address.as_str(),
                        swift_call_data,
                    )
                    .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
            }
        };

        let approval = self.check_approval(request, provider.clone()).await?;

        Ok(SwapQuoteData {
            to: MAYAN_FORWARDER_CONTRACT.to_string(),
            value: value.clone(),
            data: forwarder_call_data.encode_hex(),
            approval: approval.approval_data(),
            gas_limit: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::AssetId;

    use crate::swapper::{GemSwapMode, GemSwapOptions};

    use super::*;

    pub fn generate_mock_quote() -> Quote {
        let data = include_str!("test/quote_response.json");

        let quote: Quote = serde_json::from_str(data).expect("Failed to deserialize Quote");
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
            options: GemSwapOptions {
                slippage: 12.into(),
                fee: None,
                preferred_providers: vec![],
            },
        }
    }

    #[test]
    fn test_address_to_bytes32_valid() {
        let provider = MayanSwiftProvider::default();
        let address = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";
        let bytes32 = provider.address_to_bytes32(address).unwrap();
        let expected_bytes32 = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 85, 198, 171, 218, 94, 42, 82, 65, 170, 8, 72, 107, 213, 12, 247, 212, 117, 207, 36,
        ];
        assert_eq!(bytes32, expected_bytes32);
    }

    #[test]
    fn test_address_to_bytes32_invalid() {
        let provider = MayanSwiftProvider::default();
        let invalid_address = "invalid_address";
        let result = provider.address_to_bytes32(invalid_address);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_amount_to_wei_valid() {
        let provider = MayanSwiftProvider::default();
        let amount = 1.23;
        let result = provider.convert_amount_to_wei(amount, 18).unwrap();
        assert_eq!(result, "1230000000000000000"); // 1.23 ETH in Wei
    }

    #[test]
    fn test_convert_amount_to_wei_invalid() {
        let provider = MayanSwiftProvider::default();
        let amount = -1.0;
        let result = provider.convert_amount_to_wei(amount, 18);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_swift_order_params_valid() {
        let provider = MayanSwiftProvider::default();
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
            let provider = MayanSwiftProvider::default();
            let result = provider.get_amount_of_fractional_amount(amount, decimals);
            assert!(result.is_ok(), "Failed for amount: {}", amount);
            assert_eq!(result.unwrap(), expected.to_string());
        }
    }
}
