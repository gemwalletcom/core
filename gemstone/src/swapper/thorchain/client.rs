use super::{
    asset::THORChainAsset,
    model::{InboundAddress, QuoteSwapRequest, QuoteSwapResponse, Transaction},
};
use crate::network::{AlienHttpMethod, AlienProvider, AlienTarget, X_CACHE_TTL};
use crate::swapper::SwapperError;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct ThorChainSwapClient {
    provider: Arc<dyn AlienProvider>,
}

impl ThorChainSwapClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_quote(
        &self,
        endpoint: &str,
        from_asset: THORChainAsset,
        to_asset: THORChainAsset,
        value: String,
        streaming_interval: i64,
        streaming_quantity: i64,
        affiliate: String,
        affiliate_bps: i64,
    ) -> Result<QuoteSwapResponse, SwapperError> {
        let params = QuoteSwapRequest {
            from_asset: from_asset.asset_name(),
            to_asset: to_asset.asset_name(),
            amount: value.clone(),
            affiliate,
            affiliate_bps,
            streaming_interval,
            streaming_quantity,
        };
        let query = serde_urlencoded::to_string(params).unwrap();
        let target = AlienTarget::get(format!("{}{}?{}", endpoint, "/thorchain/quote/swap", query).as_str());

        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        serde_json::from_slice(&data).map_err(SwapperError::from)
    }

    #[allow(dead_code)]
    pub async fn get_inbound_addresses(&self, endpoint: &str) -> Result<Vec<InboundAddress>, SwapperError> {
        let target = AlienTarget {
            url: format!("{}/thorchain/inbound_addresses", endpoint),
            method: AlienHttpMethod::Get,
            headers: Some(HashMap::from([(X_CACHE_TTL.into(), "600".into())])),
            body: None,
        };

        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        serde_json::from_slice(&data).map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, endpoint: &str, transaction_hash: &str) -> Result<Transaction, SwapperError> {
        let target = AlienTarget::get(format!("{}/thorchain/tx/{}", endpoint, transaction_hash).as_str());

        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        serde_json::from_slice(&data).map_err(SwapperError::from)
    }
}
