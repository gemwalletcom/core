use super::{
    asset::THORChainAsset,
    model::{InboundAddress, QuoteSwapRequest, QuoteSwapResponse, Transaction},
};
use crate::{SwapperError, alien::X_CACHE_TTL};
use gem_client::Client;
use serde_urlencoded;
use std::{collections::HashMap, fmt::Debug};

#[derive(Clone, Debug)]
pub struct ThorChainSwapClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> ThorChainSwapClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(
        &self,
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
            amount: value,
            affiliate,
            affiliate_bps,
            streaming_interval,
            streaming_quantity,
        };
        let query = serde_urlencoded::to_string(params).map_err(SwapperError::from)?;
        let path = format!("/thorchain/quote/swap?{query}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }

    pub async fn get_inbound_addresses(&self) -> Result<Vec<InboundAddress>, SwapperError> {
        let headers = HashMap::from([(X_CACHE_TTL.to_string(), "600".to_string())]);
        self.client
            .get_with_headers("/thorchain/inbound_addresses", Some(headers))
            .await
            .map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, transaction_hash: &str) -> Result<Transaction, SwapperError> {
        let path = format!("/thorchain/tx/{transaction_hash}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}
