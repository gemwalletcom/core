use crate::network::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::swapper::models::SwapperError;
use crate::swapper::thorchain::model::{QuoteSwapRequest, QuoteSwapResponse};
use std::sync::Arc;

use super::asset::THORChainAsset;
use super::model::Transaction;

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
        affiliate: String,
        affiliate_bps: i64,
    ) -> Result<QuoteSwapResponse, SwapperError> {
        let params = QuoteSwapRequest {
            from_asset: from_asset.asset_name(),
            to_asset: to_asset.asset_name(),
            amount: value,
            affiliate,
            affiliate_bps,
            streaming_interval: 1,
            streaming_quantity: 0,
        };
        let query = serde_urlencoded::to_string(params).unwrap();
        let url = format!("{}{}?{}", endpoint, "/thorchain/quote/swap", query);

        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let result: QuoteSwapResponse = serde_json::from_slice(&data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        Ok(result)
    }

    pub async fn get_transaction_status(&self, endpoint: &str, transaction_hash: &str) -> Result<Transaction, SwapperError> {
        let target = AlienTarget {
            url: format!("{}/thorchain/tx/{}", endpoint, transaction_hash),
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let result: Transaction = serde_json::from_slice(&data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        Ok(result)
    }
}
