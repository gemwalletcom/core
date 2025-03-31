use alloy_core::{
    hex::{decode as HexDecode, ToHexExt},
    primitives::{Address, U256},
};
use async_trait::async_trait;
use bs58;
use gem_solana::WSOL_TOKEN_ADDRESS;
use num_bigint::BigInt;
use num_traits::ToBytes;
use rand::Rng;
use std::{str::FromStr, sync::Arc};

use super::{
    forwarder::MayanForwarder,
    relayer::{MayanRelayer, Quote, QuoteOptions, QuoteType, QuoteUrlParams},
    swift::{MayanSwift, OrderParams},
    MAYAN_FORWARDER_CONTRACT,
};
use crate::{
    config::swap_config::SwapReferralFee,
    network::AlienProvider,
    swapper::{
        approval::{check_approval, CheckApprovalType},
        ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData,
        SwapQuoteRequest, SwapRoute, SwapperError,
    },
};
use gem_evm::{
    ether_conv,
    mayan::deployment::{get_swift_providers, WormholeId},
};
use primitives::{AssetId, Chain, ChainType};

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
            let decoded = bs58::decode(address)
                .into_vec()
                .map_err(|_| SwapperError::InvalidAddress { address: address.to_string() })?;

            let mut bytes32 = [0u8; 32];
            if decoded.len() == 32 {
                bytes32.copy_from_slice(&decoded);
            } else {
                return Err(SwapperError::InvalidAddress {
                    address: format!("Solana address wrong length: {}", address),
                });
            }
            Ok(bytes32)
        } else {
            Ok(self.address_to_bytes32(address)?)
        }
    }

    fn build_swift_order_params(&self, request: &SwapQuoteRequest, quote: &Quote) -> Result<OrderParams, SwapperError> {
        let src_chain_id = quote.from_token.w_chain_id;
        let dest_chain_id = quote.to_token.w_chain_id;

        let deadline = quote.deadline64.parse::<u64>().map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Failed to parse deadline".to_string(),
        })?;

        let trader_address = self.to_native_wormhole_address(&request.wallet_address, src_chain_id)?;
        let destination_address = self.to_native_wormhole_address(&request.destination_address, dest_chain_id)?;
        let token_out = self.to_native_wormhole_address(&quote.to_token.contract, dest_chain_id)?;

        let min_amount_out = ether_conv::to_bn_wei(&quote.min_amount_out.to_string(), quote.to_token.decimals as u32).to_string();

        let random_bytes = Self::generate_random_bytes32();

        let referrer_fee = self.get_referrer(request)?;
        // It's always Solana address
        let referrer_addr: [u8; 32] = referrer_fee
            .as_ref()
            .map(|x| self.to_native_wormhole_address(x.address.as_str(), WormholeId::Solana as u64))
            .unwrap_or(Ok([0u8; 32]))?;
        let referrer_bps = referrer_fee.map(|x| x.bps).unwrap_or(0);

        let params = OrderParams {
            trader: trader_address,
            token_out,
            min_amount_out: min_amount_out.parse().map_err(|_| SwapperError::InvalidAmount)?,
            gas_drop: 0,
            cancel_fee: quote.cancel_relayer_fee64.unwrap_or(0),
            refund_fee: quote.refund_relayer_fee64.unwrap_or(0),
            deadline,
            dest_addr: destination_address,
            dest_chain_id: dest_chain_id.to_string().parse().map_err(|_| SwapperError::InvalidAmount)?,
            referrer_addr,
            referrer_bps: referrer_bps as u8,
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
        let addr = Address::from_str(address).map_err(|_| SwapperError::InvalidAddress { address: address.to_string() })?;
        let mut bytes32 = [0u8; 32];
        bytes32[12..].copy_from_slice(addr.as_slice());
        Ok(bytes32)
    }

    fn get_referrer(&self, request: &SwapQuoteRequest) -> Result<Option<SwapReferralFee>, SwapperError> {
        if let Some(fees) = &request.options.fee {
            return Ok(Some(fees.solana.clone()));
        }
        Ok(None)
    }

    fn normalize_address(asset_id: &AssetId) -> Result<String, SwapperError> {
        if let Some(token_id) = &asset_id.token_id {
            Ok(token_id.clone())
        } else {
            match asset_id.chain.chain_type() {
                ChainType::Ethereum => Ok(Address::ZERO.to_string()),
                ChainType::Solana => Ok(WSOL_TOKEN_ADDRESS.to_string()),
                _ => Err(SwapperError::NotSupportedAsset),
            }
        }
    }

    async fn fetch_quote_from_request(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<Quote, SwapperError> {
        let mayan_relayer = MayanRelayer::default_relayer(provider.clone());
        let referrer = self.get_referrer(request)?;

        let from_token = Self::normalize_address(&request.from_asset)?;
        let to_token = Self::normalize_address(&request.to_asset)?;

        let amount = &request.value.parse::<BigInt>()?;

        let quote_params = QuoteUrlParams::from_params(
            amount.to_string(),
            from_token,
            request.from_asset.chain,
            to_token,
            request.to_asset.chain,
            &QuoteOptions::default(),
            Some("auto".to_string()),
            referrer.clone().map(|x| x.address),
            referrer.map(|x| x.bps),
        );

        let quote = mayan_relayer.get_quote(quote_params).await.map_err(|e| SwapperError::ComputeQuoteError {
            msg: format!("Mayan relayer quote error: {:?}", e),
        })?;

        // TODO: adjust to find most effective quote
        let most_effective_quote = quote.into_iter().filter(|x| x.r#type == QuoteType::Swift.to_string()).last();

        most_effective_quote.ok_or(SwapperError::NoQuoteAvailable)
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
            SwapChainAsset::All(Chain::AvalancheC),
            SwapChainAsset::All(Chain::SmartChain),
            SwapChainAsset::All(Chain::Ethereum),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // TODO: remove this when we're ready to build solana tx
        if request.from_asset.chain.chain_type() == ChainType::Solana {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let quote = self.fetch_quote_from_request(request, provider.clone()).await?;

        let serialized_quote = serde_json::to_string(&quote).map_err(|e| SwapperError::ComputeQuoteError {
            msg: format!("Failed to serialize quote: {}", e),
        })?;

        let to_value = ether_conv::to_bn_wei(&quote.expected_amount_out.to_string(), quote.to_token.decimals as u32).to_string();
        let to_min_value = ether_conv::to_bn_wei(&quote.min_amount_out.to_string(), quote.to_token.decimals as u32).to_string();

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value,
            to_min_value,
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps: quote.slippage_bps,
                routes: vec![SwapRoute {
                    route_data: serialized_quote,
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    gas_limit: None,
                }],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let request = &quote.request;

        let mayan_quote: Quote =
            serde_json::from_str(&quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?.route_data).map_err(|e| SwapperError::ComputeQuoteError {
                msg: format!("Failed to deserialize quote: {}", e),
            })?;

        let swift_address = mayan_quote.swift_mayan_contract.ok_or(SwapperError::InvalidRoute)?;
        let swift_contract = MayanSwift::default();
        let forwarder = MayanForwarder::default();
        let swift_order_params = self.build_swift_order_params(&quote.request, &mayan_quote)?;

        let is_native = mayan_quote.swift_input_contract == Address::ZERO;
        let swift_call_data = if is_native {
            swift_contract.encode_create_order_with_eth(swift_order_params)?
        } else {
            let token_address = mayan_quote.swift_input_contract;
            let token_amount = U256::from_str(&quote.from_value).map_err(|_| SwapperError::InvalidAmount)?;

            swift_contract.encode_create_order_with_token(token_address, token_amount, swift_order_params)?
        };

        let mut value = quote.from_value.clone();
        let amount = U256::from(mayan_quote.effective_amount_in64);

        let forwarder_call_data = if mayan_quote.from_token.contract == mayan_quote.swift_input_contract.to_string() {
            if mayan_quote.from_token.contract == Address::ZERO.to_string() {
                forwarder.encode_forward_eth_call(swift_address, swift_call_data.clone())?
            } else {
                value = "0".to_string();
                let token_address = mayan_quote.swift_input_contract;

                forwarder.encode_forward_erc20_call(token_address, amount, None, swift_address, swift_call_data.clone())?
            }
        } else {
            let evm_swap_router_address = mayan_quote.evm_swap_router_address.ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing evmSwapRouterAddress".to_string(),
            })?;
            let evm_swap_router_calldata = mayan_quote.evm_swap_router_calldata.ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing evmSwapRouterCalldata".to_string(),
            })?;
            let min_middle_amount = mayan_quote.min_middle_amount.ok_or_else(|| SwapperError::ComputeQuoteError {
                msg: "Missing minMiddleAmount".to_string(),
            })?;

            let min_middle_amount_bigint = ether_conv::to_bn_wei(&min_middle_amount.to_string(), mayan_quote.swift_input_decimals as u32);
            let amount_in = U256::from_str(quote.from_value.as_str()).map_err(|_| SwapperError::InvalidAmount)?;
            let swap_data = HexDecode(evm_swap_router_calldata).map_err(SwapperError::from)?;
            let min_middle_amount = U256::from_le_slice(min_middle_amount_bigint.to_le_bytes().as_slice());

            let swap_protocol_address = Address::from_str(&evm_swap_router_address).map_err(|_| SwapperError::InvalidAddress {
                address: evm_swap_router_address.to_string(),
            })?;
            let middle_token_address = mayan_quote.swift_input_contract;
            let swift_protocol_address = swift_address;

            if mayan_quote.from_token.contract == Address::ZERO.to_string() {
                forwarder.encode_swap_and_forward_eth_call(
                    amount_in,
                    swap_protocol_address,
                    swap_data,
                    middle_token_address,
                    min_middle_amount,
                    swift_protocol_address,
                    swift_call_data,
                )?
            } else {
                value = "0".to_string();
                let token_in = mayan_quote.from_token.contract;
                let token_in_address = Address::from_str(&token_in).map_err(|_| SwapperError::InvalidAddress { address: token_in.to_string() })?;

                forwarder.encode_swap_and_forward_erc20_call(
                    token_in_address,
                    amount_in,
                    None,
                    swap_protocol_address,
                    swap_data,
                    middle_token_address,
                    min_middle_amount,
                    swift_protocol_address,
                    swift_call_data,
                )?
            }
        };

        let approval = self.check_approval(request, provider.clone()).await?;

        Ok(SwapQuoteData {
            to: MAYAN_FORWARDER_CONTRACT.to_string(),
            value,
            data: forwarder_call_data.encode_hex(),
            approval: approval.approval_data(),
            gas_limit: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::AssetId;

    use crate::swapper::{mayan::relayer::QuoteResponse, GemSlippage, GemSwapMode, GemSwapOptions, SlippageMode};

    use super::*;

    pub fn generate_mock_quote() -> Vec<Quote> {
        let data = include_str!("test/quote_response.json");
        let response: QuoteResponse = serde_json::from_str(data).expect("Failed to deserialize Quote");
        response.quotes
    }

    pub fn generate_mock_request() -> SwapQuoteRequest {
        SwapQuoteRequest {
            wallet_address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            destination_address: "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR".to_string(),
            from_asset: AssetId::from_chain(Chain::Ethereum),
            to_asset: AssetId::from_chain(Chain::Solana),
            value: "1230000000000000000".to_string(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions {
                slippage: GemSlippage {
                    bps: 100,
                    mode: SlippageMode::Auto,
                },
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
    fn test_build_swift_order_params_valid() {
        let provider = MayanSwiftProvider::default();
        let request = generate_mock_request();
        let quotes = generate_mock_quote();
        let quote = quotes.first().unwrap();
        let result = provider.build_swift_order_params(&request, quote).inspect_err(|e| {
            eprintln!("Failed to build swift order params: {}", e);
        });

        assert!(result.is_ok())
    }

    #[test]
    fn test_quote_serialization_deserialization() {
        let quotes = generate_mock_quote();
        let quote = quotes.first().unwrap();

        // Serialize the quote
        let serialized = serde_json::to_string(&quote).expect("Failed to serialize quote");

        // Deserialize the quote
        let deserialized: Quote = serde_json::from_str(&serialized).expect("Failed to deserialize quote");

        // Verify key fields match
        assert_eq!(quote.r#type, deserialized.r#type);
        assert_eq!(quote.from_token.name, deserialized.from_token.name);
        assert_eq!(quote.from_token.contract.to_string(), deserialized.from_token.contract.to_string());
        assert_eq!(quote.to_token.name, deserialized.to_token.name);
        assert_eq!(quote.to_token.contract.to_string(), deserialized.to_token.contract.to_string());
        assert_eq!(quote.min_amount_out, deserialized.min_amount_out);
        assert_eq!(quote.swift_auction_mode, deserialized.swift_auction_mode);
    }
}
