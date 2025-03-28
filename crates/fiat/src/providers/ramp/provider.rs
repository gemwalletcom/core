use primitives::{AssetId, FiatBuyQuote, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus};
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

#[async_trait]
impl FiatProvider for RampClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_buy_assets(request.clone().fiat_currency, request.clone().ip_address)
            .await?
            .assets;
        let crypto_asset_symbol = self.get_crypto_asset_symbol(request_map);

        if !assets.iter().any(|x| x.crypto_asset_symbol() == crypto_asset_symbol) {
            return Err("asset buy not supported".into());
        }

        let payload = QuoteRequest {
            crypto_asset_symbol,
            fiat_currency: request.clone().fiat_currency,
            fiat_value: Some(request.fiat_amount),
            crypto_amount: None,
        };
        let quote = self.get_client_buy_quote(payload).await?;

        Ok(self.get_fiat_buy_quote(request.clone(), quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        let assets = self
            .get_supported_sell_assets(request.clone().fiat_currency, request.clone().ip_address)
            .await?
            .assets;

        let crypto_asset_symbol = self.get_crypto_asset_symbol(request_map);
        if !assets.iter().any(|x| x.crypto_asset_symbol() == crypto_asset_symbol) {
            return Err("asset buy not supported".into());
        }

        let payload = QuoteRequest {
            crypto_asset_symbol,
            fiat_currency: request.clone().fiat_currency,
            fiat_value: None,
            crypto_amount: Some(request.clone().crypto_value),
        };
        let quote = self.get_client_sell_quote(payload).await?;

        Ok(self.get_fiat_sell_quote(request.clone(), quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_buy_assets("USD".to_string(), "127.0.0.0".to_string())
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
        let transaction_type = FiatQuoteType::Buy;

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
