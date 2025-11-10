use chrono::{Duration, Utc};
use fiat::{FiatProvider, model::FiatProviderAsset};
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{AssetTag, Diff, FiatProviderName, currency::Currency};
use storage::Database;
use storage::{AssetFilter, AssetUpdate};

pub struct FiatAssetsUpdater {
    database: Database,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatAssetsUpdater {
    pub fn new(database: Database, providers: Vec<Box<dyn FiatProvider + Send + Sync>>) -> Self {
        
        Self { database, providers }
    }

    pub async fn update_buyable_sellable_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let enabled_asset_ids = self.database.client()?.get_fiat_assets_is_enabled()?;

        // buyable
        let buyable_assets_ids = self
            .database
            .client()?
            .assets()
            .get_assets_by_filter(vec![AssetFilter::IsBuyable(true)])?
            .into_iter()
            .map(|x| x.asset.id.to_string())
            .collect::<Vec<String>>();
        let buyable_result = Diff::compare(buyable_assets_ids, enabled_asset_ids.clone());

        let _ = self
            .database
            .client()?
            .assets()
            .update_assets(buyable_result.missing.clone(), vec![AssetUpdate::IsBuyable(true)]);
        let _ = self
            .database
            .client()?
            .assets()
            .update_assets(buyable_result.different.clone(), vec![AssetUpdate::IsBuyable(false)]);

        // sellable
        let sellable_assets_ids = self
            .database
            .client()?
            .assets()
            .get_assets_by_filter(vec![AssetFilter::IsSellable(true)])?
            .into_iter()
            .filter(|x| x.score.rank > 25)
            .map(|x| x.asset.id.to_string())
            .collect::<Vec<String>>();

        let sellable_result = Diff::compare(sellable_assets_ids, enabled_asset_ids.clone());
        let _ = self
            .database
            .client()?
            .assets()
            .update_assets(sellable_result.missing.clone(), vec![AssetUpdate::IsSellable(true)]);
        let _ = self
            .database
            .client()?
            .assets()
            .update_assets(sellable_result.different.clone(), vec![AssetUpdate::IsSellable(false)]);

        Ok(1)
    }

    pub async fn update_trending_fiat_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let from = Utc::now() - Duration::days(30);
        let asset_ids = self.database.client()?.fiat().get_fiat_assets_popular(from.naive_utc(), 30)?;
        Ok(self
            .database
            .client()?
            .tag()
            .set_assets_tags_for_tag(AssetTag::TrendingFiatPurchase.as_ref(), asset_ids.clone())?)
    }

    pub async fn update_fiat_assets(&mut self, name: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut total_assets = 0;

        for i in 0..self.providers.len() {
            let provider_name = self.providers[i].name();
            match self.update_fiat_assets_for_provider(provider_name.clone()).await {
                Ok(count) => {
                    info_with_fields!(name, provider = provider_name.id(), assets = count.to_string());
                    total_assets += count;
                }
                Err(e) => error_with_fields!(name, &*e, provider = provider_name.id()),
            }
        }

        Ok(total_assets)
    }

    fn get_provider(&self, provider_name: FiatProviderName) -> Result<&(dyn FiatProvider + Send + Sync), Box<dyn std::error::Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|p| p.name() == provider_name)
            .map(|p| p.as_ref())
            .ok_or_else(|| format!("Provider {} not found", provider_name.id()).into())
    }

    async fn update_fiat_assets_for_provider(&mut self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;
        let assets = provider.get_assets().await?;
        let asset_count = assets.len();

        let validated_assets: Vec<(FiatProviderAsset, Option<primitives::AssetId>)> = assets
            .into_iter()
            .map(|fiat_asset| {
                (
                    fiat_asset.clone(),
                    fiat_asset.asset_id().filter(|id| self.database.client().ok().and_then(|mut c| c.assets().get_asset(&id.to_string()).ok()).is_some()),
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

        for asset in insert_assets {
            self.database.client()?.fiat().add_fiat_assets(vec![asset])?;
        }

        Ok(asset_count)
    }

    pub async fn update_fiat_countries(&mut self, name: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        for i in 0..self.providers.len() {
            let provider_name = self.providers[i].name();
            match self.update_fiat_countries_for_provider(provider_name.clone()).await {
                Ok(count) => info_with_fields!(name, provider = provider_name.id(), countries = count.to_string()),
                Err(e) => error_with_fields!(name, &*e, provider = provider_name.id()),
            }
        }
        Ok(true)
    }

    async fn update_fiat_countries_for_provider(&mut self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;
        let countries = provider.get_countries().await?;
        let country_count = countries.len();
        self.database
            .client()?
            .fiat()
            .add_fiat_providers_countries(countries.into_iter().map(storage::models::FiatProviderCountry::from_primitive).collect())?;
        Ok(country_count)
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
