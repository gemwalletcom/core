use primitives::{
    AssetId, FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

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
        let crypto_asset_symbol = self.get_crypto_asset_symbol(request_map);

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
        let crypto_asset_symbol = self.get_crypto_asset_symbol(request_map);

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
        let countries = self.get_countries().await?;
        let assets = self.get_supported_buy_assets("USD".to_string(), "127.0.0.0".to_string()).await?.assets;
        let assets_symbols_map = assets.clone().into_iter().map(|x| x.crypto_asset_symbol()).collect::<HashSet<String>>();

        let asset_to_countries: HashMap<String, HashMap<String, Vec<String>>> = countries.iter().fold(HashMap::new(), |mut map, country| {
            let supported_assets = &country.api_v3_supported_assets.clone().unwrap_or_default().into_iter().collect::<HashSet<_>>();
            if supported_assets.is_empty() {
                return map;
            }
            let assets_difference = assets_symbols_map.difference(supported_assets).collect::<HashSet<&String>>();
            for asset in assets_difference {
                map.entry(asset.clone()).or_default().insert(country.code.to_uppercase(), vec![]);
            }
            map
        });

        let assets = assets
            .into_iter()
            .flat_map(|x| Self::map_asset(x.clone(), asset_to_countries.get(&x.crypto_asset_symbol()).cloned()))
            .collect::<Vec<FiatProviderAsset>>();

        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME.id(),
                alpha2: x.code.to_uppercase(),
                is_allowed: x.card_payments_enabled,
            })
            .collect())
    }

    // full transaction: https://docs.ramp.network/webhooks#example-using-expressjs
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::from_value::<Webhook>(data)?.purchase;
        let asset = Self::map_asset(payload.asset.clone(), None).unwrap();
        let asset_id = AssetId::from(asset.chain.unwrap(), asset.token_id);

        // https://docs.ramp.network/sdk-reference#ramp-sale-transaction-object
        let status = match payload.status.as_str() {
            "CREATED" | "INITIALIZED" | "FIAT_PENDING" => FiatTransactionStatus::Pending,
            "RETURNED" | "EXPIRED" | "CANCELLED" | "REVIEW_REJECTED" | "FIAT_RETURNED" => FiatTransactionStatus::Failed,
            "RELEASED" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };
        // let transaction_type = match data.transacton_type.as_str() {
        //     "buy" => FiatQuoteType::Buy,
        //     "sell" => FiatQuoteType::Sell,
        //     _ => FiatQuoteType::Buy,
        // };
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
