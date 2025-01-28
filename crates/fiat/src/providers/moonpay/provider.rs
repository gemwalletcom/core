use crate::{
    error::FiatError,
    model::{FiatMapping, FiatProviderAsset},
    providers::moonpay::model::{Data, Webhook},
    FiatProvider,
};
use async_trait::async_trait;
use std::error::Error;

use super::client::MoonPayClient;
use primitives::{fiat_quote_request::FiatSellRequest, FiatTransactionType};
use primitives::{AssetId, FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus};

#[async_trait]
impl FiatProvider for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyRequest, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let ip_address_check = self.get_ip_address(request.clone().ip_address).await?;
        if !ip_address_check.is_allowed && !ip_address_check.is_buy_allowed {
            return Err(FiatError::FiatPurchaseNotAllowed.into());
        }

        let quote = self
            .get_buy_quote(request_map.symbol.to_lowercase(), request.fiat_currency.to_lowercase(), request.fiat_amount)
            .await?;

        if quote.quote_currency.not_allowed_countries.contains(&ip_address_check.alpha2) {
            return Err(FiatError::UnsupportedCountry(ip_address_check.alpha2).into());
        }

        if &ip_address_check.state == "US" && quote.quote_currency.not_allowed_us_states.contains(&ip_address_check.state) {
            return Err(FiatError::UnsupportedState(ip_address_check.state).into());
        }

        Ok(self.get_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, _request: FiatSellRequest, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        // println!("request: {:?}", request);
        // let quote = self.get_sell_quote(
        //     request_map.symbol.to_lowercase(),
        //     request.fiat_currency.to_lowercase(),
        //     request.crypto_amount,
        // ).await;
        // println!("quote: {:?}", quote);

        Err(Box::from("not supported"))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    // full transaction: https://dev.moonpay.com/reference/reference-webhooks-buy
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Data<Webhook>>(data)?;
        let asset = Self::map_asset(payload.data.currency).unwrap();
        let asset_id = AssetId::from(asset.chain.unwrap(), asset.token_id);

        let status = match payload.data.status.as_str() {
            "pending" => FiatTransactionStatus::Pending,
            "failed" => FiatTransactionStatus::Failed,
            "completed" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };

        let currency_amount = payload.data.base_currency_amount;
        let fee_provider = payload.data.fee_amount.unwrap_or_default();
        let fee_network = payload.data.network_fee_amount.unwrap_or_default();
        let fee_partner = payload.data.extra_fee_amount.unwrap_or_default();
        let fiat_amount = currency_amount + fee_provider + fee_network + fee_partner;
        let transaction_type = FiatTransactionType::Buy;

        let transaction = FiatTransaction {
            asset_id: Some(asset_id),
            transaction_type,
            symbol: asset.symbol,
            provider_id: Self::NAME.id(),
            provider_transaction_id: payload.data.id,
            status,
            fiat_amount,
            fiat_currency: payload.data.base_currency.code.to_uppercase(),
            transaction_hash: payload.data.crypto_transaction_id,
            address: payload.data.wallet_address,
            fee_provider: payload.data.fee_amount,
            fee_network: payload.data.network_fee_amount,
            fee_partner: payload.data.extra_fee_amount,
        };

        Ok(transaction)
    }
}
