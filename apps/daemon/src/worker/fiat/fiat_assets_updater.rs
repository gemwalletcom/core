use chrono::{Duration, Utc};
use fiat::{FiatProvider, model::FiatProviderAsset};
use gem_tracing::info_with_fields;
use primitives::{AssetId, AssetTag, Diff, FiatProviderName, currency::Currency};
use storage::{AssetFilter, AssetUpdate, FiatAssetFilter, FiatAssetRowsExt};
use storage::{AssetsRepository, Database, TagRepository};

#[derive(Clone, Copy)]
enum FiatAssetDirection {
    Buy,
    Sell,
}

pub struct FiatAssetsUpdater {
    database: Database,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatAssetsUpdater {
    pub fn new(database: Database, providers: Vec<Box<dyn FiatProvider + Send + Sync>>) -> Self {
        Self { database, providers }
    }

    pub async fn update_buyable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let enabled_asset_ids = self.enabled_fiat_asset_ids(FiatAssetDirection::Buy)?;
        let buyable_assets_ids = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::IsBuyable(true)])?
            .into_iter()
            .map(|x| x.asset.id)
            .collect::<Vec<AssetId>>();
        let result = Diff::compare(buyable_assets_ids, enabled_asset_ids);

        self.database.assets()?.update_assets(result.missing.clone(), vec![AssetUpdate::IsBuyable(true)])?;
        self.database.assets()?.update_assets(result.different.clone(), vec![AssetUpdate::IsBuyable(false)])?;

        Ok(result.missing.len() + result.different.len())
    }

    pub async fn update_sellable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let enabled_asset_ids = self.enabled_fiat_asset_ids(FiatAssetDirection::Sell)?;
        let sellable_assets_ids = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::IsSellable(true)])?
            .into_iter()
            .filter(|x| x.score.rank > 25)
            .map(|x| x.asset.id)
            .collect::<Vec<AssetId>>();

        let result = Diff::compare(sellable_assets_ids, enabled_asset_ids);
        self.database.assets()?.update_assets(result.missing.clone(), vec![AssetUpdate::IsSellable(true)])?;
        self.database.assets()?.update_assets(result.different.clone(), vec![AssetUpdate::IsSellable(false)])?;

        Ok(result.missing.len() + result.different.len())
    }

    fn enabled_fiat_asset_ids(&self, direction: FiatAssetDirection) -> Result<Vec<AssetId>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_assets_by_filter(Self::fiat_asset_filters(direction))?.asset_ids())
    }

    fn fiat_asset_filters(direction: FiatAssetDirection) -> Vec<FiatAssetFilter> {
        [
            FiatAssetFilter::HasAssetId,
            FiatAssetFilter::IsEnabled(true),
            FiatAssetFilter::IsEnabledByProvider(true),
            FiatAssetFilter::ProviderEnabled(true),
        ]
        .into_iter()
        .chain(match direction {
            FiatAssetDirection::Buy => [FiatAssetFilter::IsBuyEnabled(true), FiatAssetFilter::ProviderBuyEnabled(true)],
            FiatAssetDirection::Sell => [FiatAssetFilter::IsSellEnabled(true), FiatAssetFilter::ProviderSellEnabled(true)],
        })
        .collect()
    }

    pub async fn update_trending_fiat_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let from = Utc::now() - Duration::days(30);
        let mut fiat_client = self.database.fiat()?;
        let asset_ids = fiat_client.fiat().get_fiat_assets_popular(from.naive_utc(), 30)?;
        Ok(self.database.tag()?.set_assets_tags_for_tag(AssetTag::TrendingFiatPurchase.as_ref(), asset_ids)?)
    }

    fn get_provider(&self, provider_name: FiatProviderName) -> Result<&(dyn FiatProvider + Send + Sync), Box<dyn std::error::Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|p| p.name() == provider_name)
            .map(|p| p.as_ref())
            .ok_or_else(|| format!("Provider {} not found", provider_name.id()).into())
    }

    pub async fn update_fiat_assets_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;

        let payment_methods = provider.payment_methods().await;
        let payment_methods_json = serde_json::to_value(&payment_methods)?;
        self.database.fiat()?.update_fiat_provider_payment_methods(provider_name, payment_methods_json)?;

        let assets = provider.get_assets().await?;
        let asset_count = assets.len();

        let validated_assets: Vec<(FiatProviderAsset, Option<AssetId>)> = assets
            .into_iter()
            .map(|fiat_asset| {
                (
                    fiat_asset.clone(),
                    fiat_asset
                        .asset_id()
                        .filter(|id| self.database.assets().ok().and_then(|mut c| c.get_asset(id).ok()).is_some()),
                )
            })
            .collect();

        let assets = validated_assets
            .into_iter()
            .map(|(fiat_asset, asset)| self.map_fiat_asset(fiat_asset, asset))
            .collect::<Vec<primitives::FiatAsset>>();

        let insert_assets = assets
            .into_iter()
            .map(storage::models::FiatAssetRow::from_primitive)
            .collect::<Result<Vec<storage::models::FiatAssetRow>, _>>()?;

        if !insert_assets.is_empty() {
            self.database.fiat()?.add_fiat_assets(insert_assets)?;
        }

        info_with_fields!("fiat update assets", provider = provider_name.id(), assets = asset_count);

        Ok(asset_count)
    }

    pub async fn update_fiat_countries_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;
        let countries = provider.get_countries().await?;
        let country_count = countries.len();
        let country_rows = countries.into_iter().map(storage::models::FiatProviderCountryRow::from_primitive).collect::<Vec<_>>();
        self.database.fiat()?.add_fiat_providers_countries(country_rows)?;
        info_with_fields!("fiat update countries", provider = provider_name.id(), countries = country_count);
        Ok(country_count)
    }

    fn map_fiat_asset(&self, fiat_asset: FiatProviderAsset, asset_id: Option<AssetId>) -> primitives::FiatAsset {
        primitives::FiatAsset {
            id: fiat_asset.id,
            asset_id,
            provider: fiat_asset.provider,
            symbol: fiat_asset.symbol,
            network: fiat_asset.network,
            token_id: fiat_asset.token_id,
            enabled: fiat_asset.enabled,
            is_buy_enabled: fiat_asset.is_buy_enabled,
            is_sell_enabled: fiat_asset.is_sell_enabled,
            unsupported_countries: fiat_asset.unsupported_countries.unwrap_or_default(),
            buy_limits: fiat_asset.buy_limits.into_iter().filter(|x| x.currency == Currency::USD).collect::<Vec<_>>(), // stored usd only for now
            sell_limits: fiat_asset.sell_limits.into_iter().filter(|x| x.currency == Currency::USD).collect::<Vec<_>>(), // stored usd only for now
        }
    }
}
