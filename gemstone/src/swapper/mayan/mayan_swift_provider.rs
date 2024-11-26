use std::{str::FromStr, sync::Arc};

use alloy_core::{hex::ToHexExt, primitives::Address};
use alloy_primitives::U256;
use async_trait::async_trait;
use gem_evm::{
    address::EthereumAddress,
    jsonrpc::EthereumRpc,
    mayan::swift::deployment::{get_swift_deployment_by_chain, get_swift_deployment_chains},
};
use primitives::Chain;

use crate::{
    network::{jsonrpc_call, AlienProvider},
    swapper::{
        ApprovalType, FetchQuoteData, GemSwapProvider, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};

use super::{
    fee_manager::{CalcProtocolBpsParams, FeeManager},
    mayan_swift_contract::{MayanSwiftContract, MayanSwiftContractError, OrderParams},
};

#[derive(Debug)]
pub struct MayanSwiftProvider {}

impl From<MayanSwiftContractError> for SwapperError {
    fn from(err: MayanSwiftContractError) -> Self {
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

        let swift_contract = MayanSwiftContract::new(deployment.address, provider.clone(), request.from_asset.chain);

        let amount = &request.value;
        swift_contract
            .check_token_approval(&request.wallet_address, token_id, amount)
            .await
            .map_err(|e| SwapperError::ABIError { msg: e.to_string() })
    }

    async fn calculate_output_value(
        &self,
        request: &SwapQuoteRequest,
        provider: Arc<dyn AlienProvider>,
        order_params: &OrderParams,
    ) -> Result<String, SwapperError> {
        let fee_manager_address = Self::get_address_by_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let fee_manager = FeeManager::new(fee_manager_address);

        let token_out = if let Some(token_id) = &request.to_asset.token_id {
            let mut bytes = [0u8; 32];
            if let Ok(addr) = EthereumAddress::from_str(token_id) {
                bytes.copy_from_slice(&addr.bytes);
            }
            bytes
        } else {
            [0u8; 32]
        };

        let fees = fee_manager
            .calc_protocol_bps(
                request.wallet_address.clone(),
                &request.from_asset.chain,
                provider.clone(),
                CalcProtocolBpsParams {
                    amount_in: request.value.parse().map_err(|_| SwapperError::InvalidAmount)?,
                    token_in: EthereumAddress::zero(),
                    token_out: token_out.into(),
                    dest_chain: request.to_asset.chain.network_id().parse().unwrap(),
                    referrer_bps: 0,
                },
            )
            .await
            .map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;

        // TODO: do something with fees
        let output_value = U256::from_str(request.value.as_str()).map_err(|_| SwapperError::InvalidAmount)?;

        // Calculate output value with fees
        let output_value = output_value.checked_sub(U256::from(fees)).ok_or(SwapperError::ComputeQuoteError {
            msg: "Protocol fees calculation error".to_string(),
        })?;

        Ok(output_value.to_string())
    }

    fn add_referrer(&self, request: &SwapQuoteRequest, order_params: &mut OrderParams) {
        // let referrer_bps = if let Some(options) = &request.options {
        //     if let Some(ref_fees) = &options.fee {
        //         if let Ok(addr) = EthereumAddress::from_str(&ref_fees.evm.address) {
        //             order_params.referrer_addr.copy_from_slice(&addr.bytes);
        //         }
        //         ref_fees.evm.bps as u8
        //     } else {
        //         0
        //     }
        // } else {
        //     0
        // };

        // TODO: implement
    }

    fn build_swift_order_params(&self, request: &SwapQuoteRequest) -> Result<OrderParams, SwapperError> {
        let mut order_params = OrderParams {
            trader: [0u8; 32],
            token_out: [0u8; 32],
            min_amount_out: request.value.parse().map_err(|_| SwapperError::InvalidAmount)?, // TODO::
            // do i need to calculate output + fees here?
            gas_drop: 0,
            cancel_fee: 0,
            refund_fee: 0,
            deadline: (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 3600) as u64,
            dest_addr: [0u8; 32],
            dest_chain_id: request.to_asset.chain.network_id().parse().unwrap(),
            referrer_addr: [0u8; 32],
            referrer_bps: 0,
            auction_mode: 0,
            random: [0u8; 32],
        };

        // TODO: move to separated method to test
        let token_in = if request.from_asset.is_native() {
            EthereumAddress::zero()
        } else {
            EthereumAddress::from_str(request.from_asset.token_id.as_ref().ok_or(SwapperError::NotSupportedAsset)?).map_err(|_| {
                SwapperError::InvalidAddress {
                    address: request.from_asset.token_id.clone().unwrap(),
                }
            })?
        };

        if let Ok(wallet_addr) = EthereumAddress::from_str(&request.wallet_address) {
            let mut trader_bytes = [0u8; 32];
            trader_bytes[12..].copy_from_slice(&wallet_addr.bytes);
            order_params.trader.copy_from_slice(&trader_bytes);
        }

        // Set destination address
        if let Ok(dest_addr) = EthereumAddress::from_str(&request.destination_address) {
            let mut dest_bytes = [0u8; 32];
            dest_bytes[12..].copy_from_slice(&dest_addr.bytes);
            order_params.dest_addr.copy_from_slice(&dest_bytes);
        }

        // Set token_out for the destination token
        if let Some(token_id) = &request.to_asset.token_id {
            if let Ok(token_addr) = EthereumAddress::from_str(token_id) {
                let mut token_bytes = [0u8; 32];
                token_bytes[12..].copy_from_slice(&token_addr.bytes);
                order_params.token_out.copy_from_slice(&token_bytes);
            }
        }

        Ok(order_params)
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
        let swift_address = Self::get_address_by_chain(quote.request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let swift_contract = MayanSwiftContract::new(swift_address.clone(), provider.clone(), quote.request.from_asset.chain);
        let swift_order_params = self
            .build_swift_order_params(request)
            .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        let amount_in = quote.from_value.parse().map_err(|_| SwapperError::InvalidAmount)?;
        let data = if quote.request.from_asset.is_native() {
            swift_contract
                .encode_create_order_with_eth(swift_order_params, amount_in)
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        } else {
            swift_contract
                .encode_create_order_with_token(request.from_asset.token_id.as_ref().unwrap(), amount_in, swift_order_params)
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        };

        Ok(SwapQuoteData {
            to: swift_address,
            value: quote.from_value.clone(),
            data: data.encode_hex(),
        })
    }

    async fn get_transaction_status(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let receipt_call = EthereumRpc::GetTransactionReceipt(transaction_hash.to_string());

        let response = jsonrpc_call(&receipt_call, provider, &chain)
            .await
            .map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;

        let result: String = response.extract_result().map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;

        Ok(result == "0x1")
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, crate::swapper::SwapperError> {
        // Validate chain support
        if !self.supported_chains().contains(&request.from_asset.chain) {
            return Err(SwapperError::NotSupportedChain);
        }

        let swift_order_params = self
            .build_swift_order_params(request)
            .map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        // Check approvals if needed
        let approval = self.check_approval(request, provider.clone()).await?;

        // Get fee manager address for referral info
        let swift_address = Self::get_address_by_chain(request.from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let swift_contract = MayanSwiftContract::new(swift_address.clone(), provider.clone(), request.from_asset.chain);
        let amount_in = U256::from_str(&request.value).map_err(|_| SwapperError::InvalidAmount)?;
        let estimated_gas = if request.from_asset.is_native() {
            swift_contract
                .estimate_create_order_with_eth(request.wallet_address.as_str(), swift_order_params.clone(), amount_in)
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        } else {
            swift_contract
                .estimate_create_order_with_token(request.from_asset.token_id.as_ref().unwrap(), amount_in, swift_order_params.clone())
                .await
                .map_err(|e| SwapperError::ABIError { msg: e.to_string() })?
        };
        // Create route information
        let route = SwapRoute {
            route_type: "swift-order".to_string(),
            input: request
                .from_asset
                .token_id
                .clone()
                .unwrap_or_else(|| request.from_asset.chain.as_ref().to_string()),
            output: request.to_asset.token_id.clone().unwrap_or_else(|| request.to_asset.chain.as_ref().to_string()),
            fee_tier: "0".to_string(),                     // MayanSwift doesn't use fee tiers
            gas_estimate: Some(estimated_gas.to_string()), // TODO: check if this is correct
        };

        let output_value = self.calculate_output_value(request, provider.clone(), &swift_order_params).await?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: output_value,
            data: SwapProviderData {
                provider: self.provider().clone(),
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

    #[test]
    fn test_address_parameters() {
        let wallet = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";
        let destination = "0x1234567890123456789012345678901234567890";

        let request = SwapQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Base),
            to_asset: AssetId::from_chain(Chain::Ethereum),
            wallet_address: wallet.to_string(),
            destination_address: destination.to_string(),
            value: "292840000000000".to_string(),
            mode: GemSwapMode::ExactIn,
            options: None,
        };

        // Create provider and get params
        let provider = Arc::new(MockProvider::new());
        let swift_provider = MayanSwiftProvider::new();

        let params = tokio_test::block_on(swift_provider.build_swift_order_params(&request, provider)).unwrap();

        // Verify trader address (wallet)
        let wallet_addr = EthereumAddress::from_str(wallet).unwrap();
        assert_eq!(&params.trader[12..], &wallet_addr.bytes);
        assert_eq!(&params.trader[..12], &[0u8; 12]); // First 12 bytes should be zero

        // Verify destination address
        let dest_addr = EthereumAddress::from_str(destination).unwrap();
        assert_eq!(&params.dest_addr[12..], &dest_addr.bytes);
        assert_eq!(&params.dest_addr[..12], &[0u8; 12]);
    }
}
