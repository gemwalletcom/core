use super::chain_id::get_debridge_chain_id;
use crate::{
    network::AlienProvider,
    swapper::{
        approval::check_approval_erc20,
        debridge::{client::DeBridgeClient, models::*},
        models::*,
        GemSwapProvider, SwapperError,
    },
};
use alloy_primitives::U256;
use async_trait::async_trait;
use primitives::{AssetId, Chain, ChainType};
use std::str::FromStr;
use std::sync::Arc;

const ETH_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const SOL_ADDRESS: &str = "11111111111111111111111111111111";
#[allow(unused)]
const ETH_FORWARDER_ADDRESS: &str = "0x663DC15D3C1aC63ff12E45Ab68FeA3F0a883C251";
#[allow(unused)]
const ETH_DLN_SOURCE: &str = "0xeF4fB24aD0916217251F553c0596F8Edc630EB66";
#[allow(unused)]
const SOL_DLN_SOURCE: &str = "src5qyZHqTqecJV4aY6Cb6zDZLMDzrDKKezs22MPHr4";
const DEFAULT_GAS_LIMIT: &str = "100000";

#[derive(Debug)]
pub struct DeBridge {
    provider: SwapProviderType,
}

impl Default for DeBridge {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(SwapProvider::DeBridge),
        }
    }
}

impl DeBridge {
    pub fn boxed() -> Arc<dyn GemSwapProvider> {
        Arc::new(Self::default())
    }

    fn get_affiliate_info(&self, chain: Chain, options: &GemSwapOptions) -> (f64, String) {
        options.fee.clone().map_or((0.0, "".to_string()), |fees| match chain.chain_type() {
            ChainType::Ethereum => (fees.evm_bridge.bps as f64 / 100.0, fees.evm_bridge.address),
            ChainType::Solana => (fees.solana.bps as f64 / 100.0, fees.solana.address),
            _ => (0.0, "".to_string()),
        })
    }

    fn normalize_asset_id(&self, asset: &AssetId) -> String {
        if asset.is_token() {
            return asset.token_id.clone().unwrap();
        }

        match asset.chain.chain_type() {
            ChainType::Ethereum => ETH_ADDRESS.to_string(),
            ChainType::Solana => SOL_ADDRESS.to_string(),
            _ => "".to_string(),
        }
    }

    fn create_quote_request(&self, request: &SwapQuoteRequest, with_data: bool) -> Result<CreateOrderRequest, SwapperError> {
        let src_chain_id = get_debridge_chain_id(&request.from_asset.chain)?;
        let dst_chain_id = get_debridge_chain_id(&request.to_asset.chain)?;
        let src_chain_token_in = self.normalize_asset_id(&request.from_asset);
        let dst_chain_token_out = self.normalize_asset_id(&request.to_asset);
        let (affiliate_fee_percent, affiliate_fee_recipient) = self.get_affiliate_info(request.from_asset.chain, &request.options);

        Ok(CreateOrderRequest {
            src_chain_id: src_chain_id.to_string(),
            src_chain_token_in,
            src_chain_token_in_amount: request.value.clone(),
            dst_chain_id: dst_chain_id.to_string(),
            dst_chain_token_out,
            dst_chain_token_out_amount: "auto".to_string(),
            dst_chain_token_out_recipient: request.destination_address.clone(),
            src_chain_order_authority_address: if with_data { Some(request.wallet_address.clone()) } else { None },
            dst_chain_order_authority_address: if with_data { Some(request.destination_address.clone()) } else { None },
            affiliate_fee_percent,
            affiliate_fee_recipient,
        })
    }
}

#[async_trait]
impl GemSwapProvider for DeBridge {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            SwapChainAsset::All(Chain::Ethereum),
            SwapChainAsset::All(Chain::SmartChain),
            SwapChainAsset::All(Chain::Polygon),
            SwapChainAsset::All(Chain::AvalancheC),
            SwapChainAsset::All(Chain::Solana),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let from_chain = request.from_asset.chain;
        let to_chain = request.to_asset.chain;

        let quote_request = self.create_quote_request(request, false)?;
        let client = DeBridgeClient::new(provider);
        let dln_response: CreateOrderResponse = client.create_order(&quote_request).await?;

        let to_value = dln_response.estimation.dst_chain_token_out.max_theoretical_amount.clone();
        let to_min_value = dln_response.estimation.dst_chain_token_out.recommended_amount.clone();
        let slippage_bps = (dln_response.estimation.recommended_slippage * 100.0) as u32;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: to_value.to_string(),
            to_min_value: to_min_value.to_string(),
            data: SwapProviderData {
                provider: self.provider().clone(),
                slippage_bps,
                routes: vec![SwapRoute {
                    input: AssetId::from(from_chain, Some(dln_response.estimation.src_chain_token_in.address.clone())),
                    output: AssetId::from(to_chain, Some(dln_response.estimation.dst_chain_token_out.address.clone())),
                    route_data: serde_json::to_string(&dln_response).map_err(|_| SwapperError::InvalidRoute)?,
                    gas_limit: None,
                }],
            },
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let quote_dln_response = serde_json::from_str::<CreateOrderResponse>(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let from_chain = quote.request.from_asset.chain;
        let client = DeBridgeClient::new(provider.clone());
        let quote_request = self.create_quote_request(&quote.request, true)?;
        let dln_response: CreateOrderDataResponse = client.create_order(&quote_request).await?;
        let tx = dln_response.tx;
        let to = tx.to.unwrap_or_default();
        let value = tx.value.unwrap_or_default();
        let tx_data = tx.data.ok_or(SwapperError::TransactionError {
            msg: "Transaction data is missing".to_string(),
        })?;

        // FIXME Sanity check
        // 0. check new dln response amount >= quote_dln_response recommended_amount with slippage
        // 1. check if there is swap step
        // 2. validate approval spender address (dln source or forwarder address)
        // 3. validate tx data by decoding strictlySwapAndCall or createSaltedOrder (ETH for now)
        let need_check_approval = from_chain.chain_type() == ChainType::Ethereum && quote.request.from_asset.is_token();
        let approval: Option<ApprovalData> = if need_check_approval {
            let approval_type = check_approval_erc20(
                quote.request.wallet_address.clone(),
                quote.request.from_asset.token_id.clone().unwrap(),
                to.clone(),
                U256::from_str(&value).map_err(|_| SwapperError::InvalidRoute)?,
                provider.clone(),
                &from_chain,
            )
            .await?;
            approval_type.approval_data()
        } else {
            None
        };
        let gas_limit = if approval.is_some() { Some(DEFAULT_GAS_LIMIT.to_string()) } else { None };

        Ok(SwapQuoteData {
            to,
            value,
            data: tx_data,
            approval,
            gas_limit,
        })
    }

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let client = DeBridgeClient::new(provider);
        let status = client.get_order_status(transaction_hash).await?;
        Ok(matches!(status.status.as_str(), "Fulfilled" | "ClaimedUnlock"))
    }
}
