use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::FiatTransactionType;
use primitives::{AssetId, FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus};
use std::error::Error;

use super::{
    client::BanxaClient,
    model::{Asset, Webhook},
};

#[async_trait]
impl FiatProvider for BanxaClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(&self, request: FiatBuyRequest, request_map: FiatMapping) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.get_prices(&request.fiat_currency, &request_map.symbol).await?;
        let price = prices.prices.first().cloned().ok_or("No price available")?;

        Ok(self.get_fiat_quote(request, request_map, price))
    }

    async fn get_sell_quote(&self, _request: FiatBuyRequest, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Err(Box::from("not supported"))
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_buy_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
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
        let asset = Asset {
            coin_code: order.clone().coin_code,
            blockchains: vec![order.clone().blockchain.clone()],
        };
        let assset_id = Self::map_asset(asset)
            .first()
            .map(|asset| AssetId::from(asset.chain.unwrap(), asset.token_id.clone()));
        let transaction_type = FiatTransactionType::Buy;

        let transaction = FiatTransaction {
            asset_id: assset_id,
            transaction_type,
            symbol: order.coin_code,
            provider_id: Self::NAME.id(),
            provider_transaction_id: order.id,
            status,
            fiat_amount: order.fiat_amount,
            fiat_currency: order.fiat_code,
            transaction_hash: order.tx_hash,
            address: Some(order.wallet_address),
            fee_provider: order.payment_fee,
            fee_network: order.network_fee,
            fee_partner: order.merchant_fee,
        };

        Ok(transaction)
    }
}
