extern crate rocket;

use std::error::Error;

use primitives::{SwapQuoteResult, SwapQuoteRequest, SwapQuoteProtocolRequest};
use storage::DatabaseClient;
use swapper::SwapperClient;

pub struct SwapClient {
    database: DatabaseClient,
    client: SwapperClient,
}

impl SwapClient {
    pub async fn new(
        database_url: &str,
        client: SwapperClient
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
            client,
        }
    }

    pub async fn swap_quote(&mut self, request: SwapQuoteRequest) -> Result<SwapQuoteResult, Box<dyn Error + Send + Sync>> {
        let from_asset = self.database.get_asset(request.from_asset.to_string())?.as_primitive();
        let to_asset = self.database.get_asset(request.to_asset.to_string())?.as_primitive();
        
        if from_asset.chain() != to_asset.chain() {
            return Err("Cannot swap between different chains".into());
        }

        let quote_request = SwapQuoteProtocolRequest{
            from_asset: from_asset.id,
            to_asset: to_asset.id,
            wallet_address: request.wallet_address.clone(),
            from_amount: request.from_amount.clone(),
            to_amount: request.to_amount.clone(),
        };
        let quote = self.client.get_quote(quote_request).await?;

        Ok(SwapQuoteResult{quote})
    }
}