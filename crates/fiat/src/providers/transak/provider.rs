use super::client::TransakClient;
use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;
use primitives::fiat_quote_request::FiatSellRequest;
use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote, FiatTransaction};
use std::error::Error;

#[async_trait]
impl FiatProvider for TransakClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_buy_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(
                request_map.symbol.clone(),
                request.fiat_currency.clone(),
                request.fiat_amount,
                request_map.network.unwrap_or_default(),
                request.ip_address.clone(),
            )
            .await?;

        Ok(self.get_fiat_quote(request, quote))
    }

    async fn get_sell_quote(&self, _request: FiatSellRequest, _request_map: FiatMapping) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Err(Box::from("not supported"))
    }

    async fn get_assets(
        &self,
    ) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self
            .get_supported_assets()
            .await?
            .into_iter()
            .flat_map(Self::map_asset)
            .collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn webhook(
        &self,
        _data: serde_json::Value,
    ) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}
