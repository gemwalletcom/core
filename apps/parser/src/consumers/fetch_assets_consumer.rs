use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use settings_chain::ChainProviders;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, FetchAssetsPayload};
use tokio::sync::Mutex;

pub struct FetchAssetsConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
    pub providers: ChainProviders,
}

impl FetchAssetsConsumer {
    pub fn new(database: Arc<Mutex<DatabaseClient>>, providers: ChainProviders) -> Self {
        Self { database, providers }
    }
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn process(&mut self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if let Some(token_id) = payload.asset_id.token_id {
            match self.providers.get_token_data(payload.asset_id.chain, token_id.to_string()).await {
                Ok(asset) => {
                    self.database
                        .lock()
                        .await
                        .add_assets(vec![storage::models::Asset::from_primitive_default(asset)])?;
                    return Ok(1);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(0)
    }
}
