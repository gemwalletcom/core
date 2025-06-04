use std::error::Error;

use async_trait::async_trait;
use gem_chain_rpc::ChainProvider;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, FetchAssetsPayload};

pub struct FetchAssetsConsumer {
    pub database: DatabaseClient,
    pub providers: Vec<Box<dyn ChainProvider>>,
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn process(&mut self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if let Some(token_id) = payload.asset_id.token_id {
            let provider = self
                .providers
                .iter()
                .find(|x| x.get_chain() == payload.asset_id.chain)
                .ok_or("provider not found")?;

            match provider.get_token_data(token_id.to_string()).await {
                Ok(asset) => {
                    println!("assets consumer: found asset: {:?}", asset);
                    self.database.add_assets(vec![storage::models::Asset::from_primitive_default(asset)])?;
                    return Ok(1);
                }
                Err(e) => {
                    println!("assets consumer:  error: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(0)
    }
}
