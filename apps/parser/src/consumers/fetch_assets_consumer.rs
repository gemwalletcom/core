use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use cacher::CacherClient;
use settings_chain::ChainProviders;
use storage::DatabaseClient;
use streamer::{consumer::MessageConsumer, FetchAssetsPayload};
use tokio::sync::Mutex;

pub struct FetchAssetsConsumer {
    pub database: Arc<Mutex<DatabaseClient>>,
    pub providers: ChainProviders,
    pub cacher: CacherClient,
}

impl FetchAssetsConsumer {
    pub fn new(database: Arc<Mutex<DatabaseClient>>, providers: ChainProviders, cacher: CacherClient) -> Self {
        Self { database, providers, cacher }
    }
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn should_process(&mut self, payload: FetchAssetsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let key = format!("fetch_assets_{}", payload.asset_id.chain);
        self.cacher.can_process_now(&key, &payload.asset_id.to_string(), 30 * 86400).await
    }

    async fn process(&mut self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if let Some(token_id) = payload.asset_id.token_id {
            match self.providers.get_token_data(payload.asset_id.chain, token_id.to_string()).await {
                Ok(asset) => {
                    return Ok(self
                        .database
                        .lock()
                        .await
                        .add_assets(vec![storage::models::Asset::from_primitive_default(asset)])?);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(0)
    }
}
