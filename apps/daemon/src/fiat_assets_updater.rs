use fiat::model::{FiatProvider, FiatProviderAsset};
use primitives::AssetId;
use storage::database::DatabaseClient;

pub struct FiatAssetsUpdater {
    database: DatabaseClient,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatAssetsUpdater {
    pub fn new(database_url: &str, providers: Vec<Box<dyn FiatProvider + Send + Sync>>) -> Self {
        let database: DatabaseClient = DatabaseClient::new(database_url);
        Self {
            database,
            providers,
        }
    }

    pub async fn update_fiat_assets(
        &mut self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut fiat_assets: Vec<FiatProviderAsset> = Vec::new();

        for provider in self.providers.iter() {
            match provider.get_assets().await {
                Ok(assets) => {
                    println!(
                        "update_assets for provider: {}, assets: {:?}",
                        provider.name().as_str(),
                        assets.len()
                    );
                    fiat_assets.extend(assets.clone());

                    let assets = assets
                        .clone()
                        .iter()
                        .flat_map(|x| self.map_fiat_asset(provider.name().id(), x.clone()))
                        .collect::<Vec<primitives::FiatAsset>>();

                    let asset_ids = assets
                        .clone()
                        .into_iter()
                        .map(|x| x.asset_id.to_string())
                        .collect::<Vec<String>>();

                    let existing_assets_ids = self
                        .database
                        .get_assets(asset_ids)?
                        .into_iter()
                        .map(|x: storage::models::Asset| x.id)
                        .collect::<Vec<String>>();

                    let insert_assets = assets
                        .into_iter()
                        .map(storage::models::FiatAsset::from_primitive)
                        .filter(|x| existing_assets_ids.contains(&x.asset_id))
                        .collect::<Vec<storage::models::FiatAsset>>();

                    self.database.add_fiat_assets(insert_assets.clone())?;
                }
                Err(err) => {
                    println!(
                        "update_assets for provider: {}, error: {}",
                        provider.name().as_str(),
                        err
                    );
                }
            }
        }

        Ok(fiat_assets.len())
    }

    fn map_fiat_asset(
        &self,
        provider: String,
        fiat_asset: FiatProviderAsset,
    ) -> Option<primitives::FiatAsset> {
        let asset_id = match fiat_asset.clone().token_id {
            Some(token_id) => {
                let token_id = AssetId::format_token_id(fiat_asset.clone().chain, token_id)?;
                AssetId::from(fiat_asset.chain, Some(token_id))
            }
            None => AssetId::from_chain(fiat_asset.chain),
        };
        let asset = primitives::FiatAsset {
            asset_id,
            provider,
            symbol: fiat_asset.clone().symbol,
            network: fiat_asset.clone().network,
            enabled: fiat_asset.clone().enabled,
        };
        Some(asset)
    }
}
