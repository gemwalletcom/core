use fiat::{model::FiatProviderAsset, FiatProvider};
use primitives::Diff;
use storage::database::DatabaseClient;

pub struct FiatAssetsUpdater {
    database: DatabaseClient,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatAssetsUpdater {
    pub fn new(database_url: &str, providers: Vec<Box<dyn FiatProvider + Send + Sync>>) -> Self {
        let database: DatabaseClient = DatabaseClient::new(database_url);
        Self { database, providers }
    }

    pub async fn update_buyable_sellable_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let enabled_asset_ids = self.database.get_fiat_assets_is_enabled()?;

        // buyable
        let buyable_result = Diff::compare(self.database.get_assets_is_buyable()?, enabled_asset_ids.clone());

        let _ = self.database.set_assets_is_buyable(buyable_result.missing.clone(), true);
        let _ = self.database.set_assets_is_buyable(buyable_result.different.clone(), false);

        // sellable
        // let sellable_result = Diff::compare(self.database.get_assets_is_sellable()?, enabled_asset_ids.clone());
        // let _ = self.database.set_assets_is_sellable(sellable_result.missing.clone(), true);
        // let _ = self.database.set_assets_is_sellable(sellable_result.different.clone(), false);

        Ok(1)
    }

    pub async fn update_fiat_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut fiat_assets: Vec<FiatProviderAsset> = Vec::new();

        for provider in self.providers.iter() {
            match provider.get_assets().await {
                Ok(assets) => {
                    println!("update_assets for provider: {}, assets: {:?}", provider.name().id(), assets.len());
                    fiat_assets.extend(assets.clone());

                    let assets = assets
                        .clone()
                        .iter()
                        .map(|x| self.map_fiat_asset(provider.name().id(), x.clone()))
                        .collect::<Vec<primitives::FiatAsset>>();

                    let insert_assets = assets
                        .into_iter()
                        .map(storage::models::FiatAsset::from_primitive)
                        .collect::<Vec<storage::models::FiatAsset>>();

                    for asset in insert_assets.clone() {
                        match self.database.add_fiat_assets(vec![asset.clone()]) {
                            Ok(_) => {}
                            Err(err) => {
                                println!(
                                    "add_fiat_assets for provider: {}, {:?}:{:?} error: {}",
                                    provider.name().id(),
                                    asset.code,
                                    asset.asset_id,
                                    err
                                );
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("update_assets for provider: {}, error: {}", provider.name().id(), err);
                }
            }
        }

        Ok(fiat_assets.len())
    }

    fn map_fiat_asset(&self, provider: String, fiat_asset: FiatProviderAsset) -> primitives::FiatAsset {
        let asset_id = fiat_asset.asset_id();
        primitives::FiatAsset {
            id: fiat_asset.clone().id,
            asset_id,
            provider,
            symbol: fiat_asset.clone().symbol,
            network: fiat_asset.clone().network,
            token_id: fiat_asset.clone().token_id,
            enabled: fiat_asset.clone().enabled,
        }
    }
}
