extern crate rocket;

use std::error::Error;

use primitives::{
    FiatAssets, SwapMode, SwapQuoteProtocolRequest, SwapQuoteRequest, SwapQuoteResult,
};
use storage::DatabaseClient;
use swapper::SwapperClient;

pub struct SwapClient {
    database: DatabaseClient,
    client: SwapperClient,
}

impl SwapClient {
    pub async fn new(database_url: &str, client: SwapperClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, client }
    }

    fn get_quote_request(
        &mut self,
        request: SwapQuoteRequest,
    ) -> Result<SwapQuoteProtocolRequest, Box<dyn Error + Send + Sync>> {
        let from_asset = self
            .database
            .get_asset(request.from_asset.to_string())?
            .as_primitive();
        let to_asset = self
            .database
            .get_asset(request.to_asset.to_string())?
            .as_primitive();

        // if from_asset.chain() != to_asset.chain() {
        //     return Err("Cannot swap between different chains".into());
        // }

        let quote_request = SwapQuoteProtocolRequest {
            from_asset: from_asset.id,
            to_asset: to_asset.id,
            wallet_address: request.wallet_address.clone(),
            destination_address: request.destination_address.unwrap_or_default().clone(),
            amount: request.amount.clone(),
            mode: SwapMode::ExactIn,
            include_data: request.include_data,
        };

        Ok(quote_request)
    }

    pub async fn swap_quote(
        &mut self,
        request: SwapQuoteRequest,
    ) -> Result<SwapQuoteResult, Box<dyn Error + Send + Sync>> {
        let quote_request = self.get_quote_request(request)?;
        let quote = self.client.get_quote(quote_request).await?;
        Ok(SwapQuoteResult { quote })
    }

    pub async fn get_swap_assets(&mut self) -> Result<FiatAssets, Box<dyn Error>> {
        let assets = self.database.get_swap_assets()?;
        let version = self.database.get_swap_assets_version()?;

        Ok(FiatAssets {
            version: version as u32,
            asset_ids: assets,
        })
    }
}
