use chrono::Utc;
use std::collections::HashSet;
use std::error::Error;
use storage::database::prices::PriceFilter;
use storage::repositories::prices_providers_repository::PricesProvidersRepository;
use storage::repositories::prices_repository::PRIMARY_PRICE_MAX_AGE;
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database, PricesRepository};

pub struct AssetsHasPriceUpdater {
    database: Database,
}

impl AssetsHasPriceUpdater {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn update(&self) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        let cutoff = (Utc::now() - chrono::Duration::from_std(PRIMARY_PRICE_MAX_AGE)?).naive_utc();

        let enabled_providers = self
            .database
            .prices_providers()?
            .get_prices_providers()?
            .into_iter()
            .filter(|p| p.enabled)
            .map(|p| p.id.0)
            .collect::<Vec<_>>();

        let mut fresh_price_ids: HashSet<String> = HashSet::new();
        for provider in enabled_providers {
            let prices = self
                .database
                .prices()?
                .get_prices_by_filter(vec![PriceFilter::Provider(provider), PriceFilter::UpdatedAfter(cutoff)])?;
            fresh_price_ids.extend(prices.into_iter().map(|p| p.id));
        }

        let eligible: HashSet<String> = if fresh_price_ids.is_empty() {
            HashSet::new()
        } else {
            self.database
                .prices()?
                .get_prices_assets_for_price_ids(fresh_price_ids.into_iter().collect())?
                .into_iter()
                .map(|a| a.asset_id.to_string())
                .collect()
        };

        let current: HashSet<String> = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::HasPrice(true)])?
            .into_iter()
            .map(|a| a.asset.id.to_string())
            .collect();

        let additions: Vec<String> = eligible.difference(&current).cloned().collect();
        let removals: Vec<String> = current.difference(&eligible).cloned().collect();

        if !additions.is_empty() {
            self.database.assets()?.update_assets(additions.clone(), vec![AssetUpdate::HasPrice(true)])?;
        }
        if !removals.is_empty() {
            self.database.assets()?.update_assets(removals.clone(), vec![AssetUpdate::HasPrice(false)])?;
        }

        Ok((additions.len(), removals.len()))
    }
}
