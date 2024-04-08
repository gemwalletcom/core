use async_trait::async_trait;

use crate::api::AftermathApi;
use crate::models::{ExternalFee, TradeQuote, TradeQuoteResponse, TradeTx};
use primitives::{AssetId, Chain, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
use reqwest_enum::provider::{JsonProviderType, Provider};
use swap_provider::{SwapError, SwapProvider, DEFAULT_SWAP_SLIPPAGE};

pub struct AftermathProvider {
    provider: Provider<AftermathApi>,
    fee_address: String,
    fee_percentage: f32,
}

pub const PROVIDER_NAME: &str = "Aftermath";

impl AftermathProvider {
    pub fn new(fee_address: String, fee_percentage: f32) -> Self {
        let provider = Provider::<AftermathApi>::default();
        Self {
            provider,
            fee_address,
            fee_percentage,
        }
    }
}

#[async_trait]
impl SwapProvider for AftermathProvider {
    fn provider(&self) -> primitives::SwapProvider {
        PROVIDER_NAME.into()
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![Chain::Sui]
    }

    async fn get_quote(&self, request: SwapQuoteProtocolRequest) -> Result<SwapQuote, SwapError> {
        let quote = TradeQuote::from(&request, self.fee_address.clone(), self.fee_percentage);
        let response: TradeQuoteResponse = self
            .provider
            .request_json(AftermathApi::Quote(quote))
            .await?;

        let mut data: Option<SwapQuoteData> = None;
        if request.include_data {
            let tx = TradeTx::from(
                &response,
                request.wallet_address.clone(),
                DEFAULT_SWAP_SLIPPAGE,
            );
            let tx_response: String = self.provider.request_json(AftermathApi::Tx(tx)).await?;
            data = Some(SwapQuoteData {
                to: "".into(), // tx is programmable tx, there is no single to address
                value: response.coin_out.amount.replace('n', ""),
                data: tx_response,
            });
        }

        Ok(SwapQuote {
            chain_type: request.from_asset.chain.chain_type(),
            from_amount: response.coin_in.amount.replace('n', ""),
            to_amount: response.coin_out.amount.replace('n', ""),
            fee_percent: self.fee_percentage,
            provider: self.provider(),
            data,
        })
    }
}

fn get_coin_type(asset_id: &AssetId) -> String {
    if let Some(asset) = &asset_id.token_id {
        return asset.clone();
    }
    asset_id.chain.as_denom().unwrap_or_default().to_string()
}

impl TradeQuote {
    pub fn from(
        request: &SwapQuoteProtocolRequest,
        fee_address: String,
        fee_percentage: f32,
    ) -> Self {
        TradeQuote {
            coin_in_type: get_coin_type(&request.from_asset),
            coin_out_type: get_coin_type(&request.to_asset),
            coin_in_amount: request.amount.clone(),
            external_fee: ExternalFee {
                recipient: fee_address.clone(),
                fee_percentage,
            },
        }
    }
}

impl TradeTx {
    pub fn from(response: &TradeQuoteResponse, wallet_address: String, slippage: f32) -> Self {
        TradeTx {
            wallet_address,
            complete_route: response.clone(),
            slippage,
            is_sponsored_tx: false,
        }
    }
}
