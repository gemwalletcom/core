use primitives::{AssetId, FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus, FiatTransactionType};
use std::error::Error;

use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};

use super::{
    client::RampClient,
    model::{QuoteRequest, Webhook},
};
use async_trait::async_trait;
use primitives::fiat_quote_request::FiatSellRequest;

#[async_trait]
impl FiatProvider for RampClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyRequest, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets(request.clone().fiat_currency, request.clone().ip_address)
            .await?
            .assets;

        let crypto_asset_symbol = format!("{}_{}", request_map.network.unwrap_or_default(), request_map.symbol,);

        if !assets.iter().any(|x| x.crypto_asset_symbol() == crypto_asset_symbol) {
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

    async fn get_sell_quote(&self, _request: FiatSellRequest, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Err(Box::from("not supported"))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
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
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Webhook>(data)?.purchase;
        let asset = Self::map_asset(payload.asset.clone()).unwrap();
        let asset_id = AssetId::from(asset.chain.unwrap(), asset.token_id);

        // https://docs.ramp.network/sdk-reference#ramp-sale-transaction-object
        let status = match payload.status.as_str() {
            "CREATED" | "INITIALIZED" | "FIAT_PENDING" => FiatTransactionStatus::Pending,
            "RETURNED" | "EXPIRED" | "CANCELLED" | "REVIEW_REJECTED" | "FIAT_RETURNED" => FiatTransactionStatus::Failed,
            "RELEASED" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };
        let transaction_type = FiatTransactionType::Buy;

        let transaction = FiatTransaction {
            asset_id: Some(asset_id),
            transaction_type,
            symbol: asset.symbol,
            provider_id: Self::NAME.id(),
            provider_transaction_id: payload.purchase_view_token,
            status,
            fiat_amount: payload.fiat_value,
            fiat_currency: payload.fiat_currency,
            transaction_hash: payload.final_tx_hash,
            address: payload.receiver_address,
            fee_provider: Some(payload.base_ramp_fee),
            fee_network: Some(payload.network_fee),
            fee_partner: Some(payload.host_fee_cut),
        };
        Ok(transaction)
    }
}
