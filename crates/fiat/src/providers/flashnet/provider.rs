use std::error::Error;

use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, PaymentType};
use streamer::FiatWebhook;

use crate::FiatProvider;
use crate::model::{FiatMapping, FiatProviderAsset};
use crate::provider::generate_quote_id;

use super::{
    client::FlashnetClient,
    mapper::{map_amount, map_assets, map_crypto_amount, map_redirect_url, map_source_amount, map_webhook},
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
        Ok(map_assets(
            routes
                .routes
                .into_iter()
                .filter(|route| route.source_chain == "lightning" && route.source_asset == "BTC")
                .collect(),
        ))
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FiatProviderCountry {
            provider: Self::NAME,
            alpha2: "US".to_string(),
            is_allowed: true,
        }])
    }

    async fn process_webhook(&self, data: serde_json::Value) -> Result<FiatWebhook, Box<dyn Error + Send + Sync>> {
        let payload = serde_json::from_value::<FlashnetWebhookPayload>(data)?;
        Ok(map_webhook(payload)?)
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let chain = FiatMapping::get_network(request_map.asset_symbol.network)?;
        let symbol = request_map.asset_symbol.symbol;
        let amount = map_source_amount(request.amount);
        let estimate = self.get_estimate(&chain, &symbol, &amount).await?;
        let crypto_amount = map_crypto_amount(&estimate.estimated_out, request_map.asset.decimals as u32)?;

        Ok(FiatQuoteResponse::new(generate_quote_id(), request.amount, crypto_amount))
    }

    async fn get_quote_sell(&self, _request: FiatQuoteRequest, _request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        Err("not implemented".into())
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let network = FiatMapping::get_network(data.asset_symbol.network.clone())?;
        let amount = map_amount(data.quote.crypto_amount, data.quote.asset.decimals as u32);

        let request = FlashnetOnrampRequest {
            destination_chain: network,
            destination_asset: data.asset_symbol.symbol.clone(),
            recipient_address: data.wallet_address.clone(),
            amount,
            amount_mode: "exact_out".to_string(),
            affiliate_id: self.affiliate_id.clone(),
        };
        let response = self.create_onramp(request, &data.quote.id).await?;

        Ok(FiatQuoteUrl {
            redirect_url: map_redirect_url(&response),
            provider_transaction_id: Some(response.order_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::flashnet::model::{FlashnetEstimateResponse, FlashnetRoutesResponse};

    #[test]
    fn map_redirect_url_returns_cash_app_link() {
        let response = serde_json::from_str(include_str!("../../../testdata/flashnet/onramp_response.json")).unwrap();
        let result = map_redirect_url(&response);

        assert_eq!(result, "https://orchestration.flashnet.xyz/pay/zimH6K-d");
    }

    #[test]
    fn map_estimate_includes_affiliate_fees() {
        let response: FlashnetEstimateResponse = serde_json::from_str(include_str!("../../../testdata/flashnet/estimate.json")).unwrap();

        assert_eq!(response.estimated_out, "98951");
        assert_eq!(response.app_fees.len(), 1);
        assert_eq!(response.app_fees[0].affiliate_id, "gemwallet");
        assert_eq!(response.app_fees[0].fee_bps, 100);
    }

    #[test]
    fn map_assets_maps_supported_routes() {
        let response: FlashnetRoutesResponse = serde_json::from_str(include_str!("../../../testdata/flashnet/routes.json")).unwrap();
        let assets = map_assets(
            response
                .routes
                .into_iter()
                .filter(|route| route.source_chain == "lightning" && route.source_asset == "BTC")
                .collect(),
        );

        assert_eq!(assets.len(), 3);
        assert!(assets.iter().any(|asset| asset.id == "btc_bitcoin"));
        assert!(assets.iter().any(|asset| asset.id == "eth_base"));
        assert!(assets.iter().any(|asset| asset.id == "usdc_solana"));
    }
}
