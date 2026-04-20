use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use gem_tracing::info_with_fields;
use settings_chain::ChainProviders;
use storage::{AssetsRepository, Database};
use streamer::{FetchAssetsPayload, consumer::MessageConsumer};

pub struct FetchAssetsConsumer {
    pub database: Database,
    pub providers: ChainProviders,
    pub cacher: CacherClient,
}

#[async_trait]
impl MessageConsumer<FetchAssetsPayload, usize> for FetchAssetsConsumer {
    async fn should_process(&self, payload: FetchAssetsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchAssets(&payload.asset_id.to_string())).await
    }

    async fn process(&self, payload: FetchAssetsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let Some(token_id) = payload.asset_id.token_id.clone() else {
            return Ok(0);
        };
        let asset = self.providers.get_token_data(payload.asset_id.chain, token_id.to_string()).await?;
        let added = self.database.assets()?.add_assets(vec![asset.as_basic_primitive()])?;
        let name = format!("{:?}", asset.name);
        info_with_fields!(
            "fetch asset",
            chain = payload.asset_id.chain.as_ref(),
            symbol = asset.symbol.as_str(),
            name = name.as_str(),
            added = added
        );
        Ok(added)
    }
}
