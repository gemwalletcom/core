use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::{FiatBuyQuote, FiatQuoteType, FiatSellQuote};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus};
use std::error::Error;

use super::{
    client::BanxaClient,
    model::{Webhook, ORDER_TYPE_SELL},
};

#[async_trait]
impl FiatProvider for BanxaClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_quote_buy(
                &request_map.clone().symbol,
                &request_map.clone().network.unwrap_or_default(),
                &request.fiat_currency,
                request.fiat_amount,
            )
            .await?;

        Ok(self.get_fiat_buy_quote(request, request_map, quote))
    }

    async fn get_sell_quote(&self, request: FiatSellQuote, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        // v2/payment-methods/sell
        let method = self
            .get_payment_methods(ORDER_TYPE_SELL)
            .await?
            .into_iter()
            .find(|x| x.supported_fiats.contains(&request.fiat_currency))
            .ok_or("Payment method not found")?;

        let quote = self
            .get_quote_sell(
                &method.id,
                &request_map.symbol,
                &request_map.clone().network.unwrap_or_default(),
                &request.fiat_currency,
                request.crypto_amount,
            )
            .await?;
        Ok(self.get_fiat_sell_quote(request, request_map, quote))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_assets_buy()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
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
                alpha2: x.id,
                is_allowed: true,
            })
            .collect())
    }

    // https://docs.banxa.com/docs/webhooks
    async fn webhook(&self, data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let data = serde_json::from_value::<Webhook>(data)?;
        let order = self.get_order(&data.order_id).await?;

        // https://docs.banxa.com/docs/order-status
        let status = match order.status.as_str() {
            "pendingPayment" | "waitingPayment" | "paymentReceived" | "inProgress" | "coinTransferred" | "cryptoTransferred" | "extraVerification" => {
                FiatTransactionStatus::Pending
            }
            "cancelled" | "declined" | "expired" | "refunded" => FiatTransactionStatus::Failed,
            "complete" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
            _ => FiatTransactionStatus::Unknown,
        };
        // TODO: Add order.crypto to asset mapping
        //let assset_id = Self::map_asset(asset)
        //    .first()
        //    .map(|asset| AssetId::from(asset.chain.unwrap(), asset.token_id.clone()));

        let transaction_type = match order.order_type.as_str() {
            "BUY" => FiatQuoteType::Buy,
            "SELL" => FiatQuoteType::Sell,
            _ => FiatQuoteType::Buy,
        };

        let transaction = FiatTransaction {
            asset_id: None,
            transaction_type,
            symbol: order.crypto.id,
            provider_id: Self::NAME.id(),
            provider_transaction_id: order.id,
            status,
            fiat_amount: order.fiat_amount,
            fiat_currency: order.fiat,
            transaction_hash: order.tx_hash,
            address: Some(order.wallet_address),
            fee_provider: None,
            fee_network: order.network_fee,
            fee_partner: order.processing_fee,
        };

        Ok(transaction)
    }
}
