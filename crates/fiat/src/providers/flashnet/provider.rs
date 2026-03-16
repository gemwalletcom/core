use std::error::Error;

use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatTransaction, PaymentType};
use streamer::FiatWebhook;
use uuid::Uuid;

use crate::FiatProvider;
use crate::model::{FiatMapping, FiatProviderAsset};

use super::{
    client::FlashnetClient,
    mapper::{map_amount, map_assets, map_crypto_amount, map_order, map_redirect_url, map_source_amount},
    model::{FlashnetOnrampRequest, FlashnetWebhookPayload},
};

#[async_trait]
impl FiatProvider for FlashnetClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn payment_methods(&self) -> Vec<PaymentType> {
        vec![PaymentType::CashApp]
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn Error + Send + Sync>> {
        let routes = self.get_routes().await?;
        Ok(map_assets(routes.routes))
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FiatProviderCountry {
            provider: Self::NAME.id(),
            alpha2: "US".to_string(),
            is_allowed: true,
        }])
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransaction, Box<dyn Error + Send + Sync>> {
        let response = self.get_order_status(order_id).await?;
        Ok(map_order(response.order))
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn Error + Send + Sync>> {
        let payload = serde_json::from_value::<FlashnetWebhookPayload>(data)?;
        if !payload.event.starts_with("order.") {
            return Ok(FiatWebhook::None);
        }
        Ok(FiatWebhook::OrderId(payload.data.id))
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let chain = FiatMapping::get_network(request_map.asset_symbol.network)?;
        let symbol = request_map.asset_symbol.symbol;
        let amount = map_source_amount(request.amount);
        let estimate = self.get_estimate(&chain, &symbol, &amount).await?;
        let crypto_amount = map_crypto_amount(&estimate.estimated_out, request_map.asset.decimals as u32);

        Ok(FiatQuoteResponse::new(Uuid::new_v4().to_string(), request.amount, crypto_amount))
    }

    async fn get_quote_sell(&self, _request: FiatQuoteRequest, _request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        Err("not implemented".into())
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let network = FiatMapping::get_network(data.asset_symbol.network.clone())?;
        let amount = map_amount(data.quote.crypto_amount, data.quote.asset.decimals as u32);

        let request = FlashnetOnrampRequest {
            destination_chain: network,
            destination_asset: data.asset_symbol.symbol.to_ascii_uppercase(),
            recipient_address: data.wallet_address.clone(),
            amount,
            amount_mode: "exact_out".to_string(),
            affiliate_id: self.affiliate_id.clone(),
        };
        let response = self.create_onramp(request, &data.quote.id).await?;

        Ok(FiatQuoteUrl {
            redirect_url: map_redirect_url(response),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::flashnet::model::{FlashnetEstimateResponse, FlashnetOrder, FlashnetRoutesResponse};
    use primitives::Chain;
    use primitives::currency::Currency;

    #[test]
    fn map_assets_maps_supported_routes() {
        let response: FlashnetRoutesResponse = serde_json::from_str(include_str!("../../../testdata/flashnet/routes.json")).unwrap();
        let assets = map_assets(response.routes);

        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].chain, Some(Chain::Solana));
        assert_eq!(assets[0].symbol, "USDC");
        assert_eq!(assets[0].token_id, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()));
        assert_eq!(assets[1].chain, Some(Chain::Base));
        assert_eq!(assets[1].symbol, "USDC");
        assert!(assets.iter().all(|asset| asset.provider == FiatProviderName::Flashnet));
        assert!(assets.iter().all(|asset| !asset.is_sell_enabled));
    }

    #[test]
    fn map_redirect_url_returns_cash_app_link() {
        let response = serde_json::from_str(include_str!("../../../testdata/flashnet/onramp_response.json")).unwrap();
        let result = map_redirect_url(response);

        assert_eq!(result, "https://orchestration.flashnet.xyz/pay/zimH6K-d");
    }

    #[test]
    fn map_order_maps_completed_order() {
        let order: FlashnetOrder = serde_json::from_str(include_str!("../../../testdata/flashnet/order_completed.json")).unwrap();
        let transaction = map_order(order);

        assert_eq!(transaction.provider_id, "flashnet");
        assert_eq!(transaction.provider_transaction_id, "ord_123");
        assert_eq!(transaction.symbol, "USDC");
        assert_eq!(transaction.fiat_currency, Currency::USD.as_ref());
        assert_eq!(transaction.fiat_amount, 1.2345);
        match transaction.status {
            primitives::FiatTransactionStatus::Complete => {}
            status => panic!("Expected complete status, got {:?}", status),
        }
        assert_eq!(transaction.transaction_hash.as_deref(), Some("solana_sig_123"));
        assert_eq!(transaction.asset_id.as_ref().map(ToString::to_string).as_deref(), Some("solana"));
    }

    #[test]
    fn map_estimate_includes_affiliate_fees() {
        let response: FlashnetEstimateResponse = serde_json::from_str(include_str!("../../../testdata/flashnet/estimate.json")).unwrap();

        assert_eq!(response.estimated_out, "98951");
        assert_eq!(response.app_fees.len(), 1);
        assert_eq!(response.app_fees[0].affiliate_id, "gemwallet");
        assert_eq!(response.app_fees[0].fee_bps, 100);
    }
}
