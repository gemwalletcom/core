use std::error::Error;

use async_trait::async_trait;
use cacher::CacherClient;
use settings_chain::ChainProviders;
use storage::Database;
use streamer::{FetchAssetsPayload, consumer::MessageConsumer};

pub struct FetchAssetsConsumer {
    pub database: Database,
    pub providers: ChainProviders,
    pub cacher: CacherClient,
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn should_process(&self, payload: FetchAssetsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_now(&format!("fetch_assets:{}", payload.asset_id), 30 * 86400).await
    }

    async fn process(&self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if let Some(token_id) = payload.asset_id.token_id {
            let asset = self.providers.get_token_data(payload.asset_id.chain, token_id.to_string()).await?;
            return Ok(self.database.client()?.assets().add_assets(vec![asset.as_basic_primitive()])?);
        }
        Ok(0)
    }
}
