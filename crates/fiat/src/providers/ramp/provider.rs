use primitives::{
    AssetId, FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus,
    NumberFormatter,
};
use url::form_urlencoded::parse;

use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};

use async_trait::async_trait;

use super::{
    client::RampClient,
    model::{QuoteRequest, Webhook},
};

#[async_trait]
impl FiatProvider for RampClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets(request.clone().fiat_currency, request.clone().ip_address)
            .await?
            .assets;

        let crypto_asset_symbol = format!(
            "{}_{}",
            request_map.network.unwrap_or_default(),
            request_map.symbol,
        );

        if !assets
            .iter()
            .any(|x| x.crypto_asset_symbol() == crypto_asset_symbol)
        {
            return Err("asset not supported".into());
        }

        let payload = QuoteRequest {
            crypto_asset_symbol,
            fiat_currency: request.clone().fiat_currency,
            fiat_value: request.fiat_amount,
        };
        let quote = self.get_client_quote(payload).await?;

        Ok(self.get_fiat_quote(request.clone(), quote))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets("USD".to_string(), "127.0.0.0".to_string())
            .await?
            .assets
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    // full transaction: https://docs.ramp.network/webhooks#example-using-expressjs
    async fn webhook(
        &self,
        data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Webhook>(data)?.payload;
        let asset = Self::map_asset(payload.crypto.asset_info.clone()).unwrap();
        let asset_id = AssetId::from(asset.chain, asset.token_id);

        // https://docs.ramp.network/sdk-reference#ramp-sale-transaction-object
        let status = match payload.fiat.status.as_str() {
            "not-started" | "initiated" | "delayed" => FiatTransactionStatus::Pending,
            "failed" => FiatTransactionStatus::Failed,
            "completed" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };

        let fiat_amount = NumberFormatter::value(
            payload.crypto.amount.as_str(),
            payload.crypto.asset_info.decimals as i32,
        )
        .unwrap_or_default()
        .parse::<f64>()?;

        let transaction = FiatTransaction {
            asset_id: Some(asset_id),
            symbol: asset.symbol,
            provider_id: Self::NAME.id(),
            transaction_id: payload.id,
            status,
            fiat_amount,
            fiat_currency: payload.fiat.currency_symbol,
        };

        Ok(transaction)
    }
}
