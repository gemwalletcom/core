use crate::{
    model::{FiatMapping, FiatProviderAsset},
    FiatProvider,
};
use async_trait::async_trait;

use primitives::{FiatBuyRequest, FiatProviderName, FiatQuote};

use super::client::MoonPayClient;

#[async_trait]
impl FiatProvider for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_quote(
        &self,
        request: FiatBuyRequest,
        request_map: FiatMapping,
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let ip_address_check = self.get_ip_address(request.clone().ip_address).await?;
        if !ip_address_check.is_allowed && !ip_address_check.is_buy_allowed {
            return Err("purchase is not allowed".into());
        }

        let quote = self
            .get_buy_quote(
                request_map.symbol.to_lowercase(),
                request.fiat_currency.to_lowercase(),
                request.fiat_amount,
            )
            .await?;

        Ok(self.get_fiat_quote(request, quote))
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
}
