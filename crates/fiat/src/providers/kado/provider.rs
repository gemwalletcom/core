use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::fiat_quote_request::FiatSellRequest;
use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus};
use std::error::Error;

use super::{client::KadoClient, model::Webhook};

#[async_trait]
impl FiatProvider for KadoClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyRequest, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                request.fiat_currency.clone(),
                request_map.symbol.clone(),
                request.fiat_amount,
                request_map.network.clone().unwrap_or_default(),
            )
            .await?;

        Ok(self.get_fiat_quote(request, request_map.clone(), quote.quote))
    }

    async fn get_sell_quote(&self, _request: FiatSellRequest, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Err(Box::from("not supported"))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_blockchains()
            .await?
            .into_iter()
            .flat_map(Self::map_blockchain)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let data = serde_json::from_value::<Webhook>(data)?;
        let status = match data.webhook_type.as_str() {
            "order_pending" | "order_processing" => FiatTransactionStatus::Pending,
            "order_completed" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };

        let transaction = FiatTransaction {
            asset_id: None,
            symbol: data.data.receive_amount.unit.unwrap_or_default(),
            provider_id: Self::NAME.id(),
            provider_transaction_id: data.data.id,
            status,
            fiat_amount: data.data.buy_amount.amount.unwrap_or_default(),
            fiat_currency: data.data.currency_type,
            transaction_hash: Some(data.data.tx_hash),
            address: Some(data.data.wallet_address),
            fee_provider: data.data.processing_fee.amount,
            fee_network: data.data.gas_fee.amount,
            fee_partner: None,
        };

        Ok(transaction)
    }
}
