use async_trait::async_trait;
use primitives::{
    AssetId, FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction, FiatTransactionStatus,
};

use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};

use super::{
    client::BanxaClient,
    model::{Asset, OrderRequest, Webhook},
};

#[async_trait]
impl FiatProvider for BanxaClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let account_reference = format!("{}-{}", request.asset_id, request.wallet_address);
        let order_request = OrderRequest {
            account_reference,
            source: request.fiat_currency.to_string().clone(),
            source_amount: request.fiat_amount.to_string(),
            target: request_map.symbol.clone(),
            blockchain: request_map.network.clone().unwrap_or_default(),
            wallet_address: request.wallet_address.to_string().clone(),
            return_url_on_success: "https://gemwallet.com".to_string(),
        };
        let (prices, quote) = tokio::try_join!(
            self.get_prices(&request.fiat_currency, &request_map.symbol),
            self.get_quote_buy(order_request)
        )?;
        let price = prices.prices.first().unwrap().clone();

        Ok(self.get_fiat_quote(request, price, quote))
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

    // https://docs.banxa.com/docs/webhooks
    async fn webhook(
        &self,
        data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let data = serde_json::from_value::<Webhook>(data)?;
        let order = self.get_order(&data.order_id).await?;

        // https://docs.banxa.com/docs/order-status
        let status = match data.status.as_str() {
            "pendingPayment" | "waitingPayment" | "paymentReceived" | "inProgress"
            | "coinTransferred" | "cryptoTransferred" | "extraVerification" => {
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

        let transaction = FiatTransaction {
            asset_id: assset_id,
            symbol: order.coin_code,
            provider_id: Self::NAME.id(),
            provider_transaction_id: order.id,
            status,
            fiat_amount: order.fiat_amount,
            fiat_currency: order.fiat_code,
            transaction_hash: order.tx_hash,
            address: Some(order.wallet_address),
            fee_provider: order.fee.unwrap_or_default(),
            fee_network: order.network_fee.unwrap_or_default(),
            fee_partner: order.merchant_fee.unwrap_or_default(),
        };

        Ok(transaction)
    }
}
