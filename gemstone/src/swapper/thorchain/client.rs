use crate::network::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::swapper::thorchain::model::{QuoteSwapRequest, QuoteSwapResponse};
use crate::swapper::SwapperError;
use num_bigint::BigInt;
use std::str::FromStr;
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
        let input_amount = BigInt::from_str(&value).map_err(|_| SwapperError::InvalidAmount)?;
        let recommended_min_amount = BigInt::from_str(&result.recommended_min_amount_in).map_err(|_| SwapperError::InvalidAmount)?;
        if recommended_min_amount > input_amount {
            return Err(SwapperError::InputAmountTooSmall);
        }

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
