use crate::{
    model::{FiatMapping, FiatProviderAsset},
    providers::moonpay::model::{Data, Webhook},
    FiatProvider,
};
use async_trait::async_trait;

use primitives::{
    AssetId, FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus,
};

use super::client::MoonPayClient;

#[async_trait]
impl FiatProvider for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let ip_address_check = self.get_ip_address(request.clone().ip_address).await?;
        if !ip_address_check.is_allowed && !ip_address_check.is_buy_allowed {
            return Err("purchase is not allowed".into());
        }

        let quote = self
            .get_buy_quote(
                request_map.symbol.to_lowercase(),
                request.fiat_currency.to_lowercase(),
                request.fiat_amount,
            )
            .await?;

        Ok(self.get_fiat_quote(request, quote))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    // full transaction: https://dev.moonpay.com/reference/reference-webhooks-buy
    async fn webhook(
        &self,
        data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Data<Webhook>>(data)?;
        let asset = Self::map_asset(payload.data.currency).unwrap();
        let asset_id = AssetId::from(asset.chain, asset.token_id);

        let status = match payload.transaction_type.as_str() {
            "transaction_created" => FiatTransactionStatus::Pending,
            "transaction_failed" => FiatTransactionStatus::Failed,
            "transaction_updated" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };

        let transaction = FiatTransaction {
            asset_id: Some(asset_id),
            symbol: asset.symbol,
            provider_id: Self::NAME.id(),
            transaction_id: payload.data.id,
            status,
            fiat_amount: payload.data.base_currency_amount,
            fiat_currency: payload.data.base_currency.code.to_uppercase(),
        };

        Ok(transaction)
    }
}
