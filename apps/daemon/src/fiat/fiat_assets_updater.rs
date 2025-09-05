use chrono::{Duration, Utc};
use fiat::{model::FiatProviderAsset, FiatProvider};
use primitives::{currency::Currency, AssetTag, Diff};
use storage::{AssetFilter, AssetUpdate, AssetsRepository, DatabaseClient};

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
        let buyable_assets_ids = self
            .database
            .assets()
            .get_assets_by_filter(vec![AssetFilter::IsBuyable(true)])?
            .into_iter()
            .map(|x| x.asset.id.to_string())
            .collect::<Vec<String>>();
        let buyable_result = Diff::compare(buyable_assets_ids, enabled_asset_ids.clone());

        let _ = self
            .database
            .assets()
            .update_assets(buyable_result.missing.clone(), vec![AssetUpdate::IsBuyable(true)]);
        let _ = self
            .database
            .assets()
            .update_assets(buyable_result.different.clone(), vec![AssetUpdate::IsBuyable(false)]);

        // sellable
        let sellable_assets_ids = self
            .database
            .assets()
            .get_assets_by_filter(vec![AssetFilter::IsSellable(true)])?
            .into_iter()
            .filter(|x| x.score.rank > 25)
            .map(|x| x.asset.id.to_string())
            .collect::<Vec<String>>();

        let sellable_result = Diff::compare(sellable_assets_ids, enabled_asset_ids.clone());
        let _ = self
            .database
            .assets()
            .update_assets(sellable_result.missing.clone(), vec![AssetUpdate::IsSellable(true)]);
        let _ = self
            .database
            .assets()
            .update_assets(sellable_result.different.clone(), vec![AssetUpdate::IsSellable(false)]);

        Ok(1)
    }

    pub async fn update_trending_fiat_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let from = Utc::now() - Duration::days(30);
        let asset_ids = self.database.fiat().get_fiat_assets_popular(from.naive_utc(), 30)?;
        self.database
            .tag()
            .set_assets_tags_for_tag(AssetTag::TrendingFiatPurchase.as_ref(), asset_ids.clone())
    }

    pub async fn update_fiat_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut fiat_assets: Vec<FiatProviderAsset> = Vec::new();

        for provider in self.providers.iter() {
            match provider.get_assets().await {
                Ok(assets) => {
                    println!("update_assets for provider: {}, assets: {:?}", provider.name().id(), assets.len());
                    fiat_assets.extend(assets.clone());

                    let validated_assets: Vec<(FiatProviderAsset, Option<primitives::AssetId>)> = assets
                        .into_iter()
                        .map(|fiat_asset| {
                            (
                                fiat_asset.clone(),
                                fiat_asset.asset_id().filter(|id| self.database.get_asset(&id.to_string()).is_ok()),
                            )
                        })
                        .collect();

                    let assets = validated_assets
                        .into_iter()
                        .map(|(fiat_asset, asset)| self.map_fiat_asset(fiat_asset, asset))
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

    pub async fn update_fiat_providers_countries(&mut self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        for provider in self.providers.iter() {
            match provider.get_countries().await {
                Ok(countries) => {
                    println!("update_countries for provider: {}, countries: {:?}", provider.name().id(), countries.len());
                    let _ = self
                        .database
                        .add_fiat_providers_countries(countries.into_iter().map(storage::models::FiatProviderCountry::from_primitive).collect());
                }
                Err(err) => {
                    println!("update_countries for provider: {}, error: {}", provider.name().id(), err);
                }
            }
        }
        Ok(true)
    }

    fn map_fiat_asset(&self, fiat_asset: FiatProviderAsset, asset_id: Option<primitives::AssetId>) -> primitives::FiatAsset {
        primitives::FiatAsset {
            id: fiat_asset.id,
            asset_id,
            provider: fiat_asset.provider.id(),
            symbol: fiat_asset.symbol,
            network: fiat_asset.network,
            token_id: fiat_asset.token_id,
            enabled: fiat_asset.enabled,
            unsupported_countries: fiat_asset.unsupported_countries.unwrap_or_default(),
            buy_limits: fiat_asset.buy_limits.into_iter().filter(|x| x.currency == Currency::USD).collect::<Vec<_>>(), // stored usd only for now
            sell_limits: fiat_asset.sell_limits.into_iter().filter(|x| x.currency == Currency::USD).collect::<Vec<_>>(), // stored usd only for now
        }
    }
}
