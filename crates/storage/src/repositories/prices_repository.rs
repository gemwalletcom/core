use crate::{DatabaseError, DieselResultExt};
use chrono::Utc;
use primitives::{AssetId, AssetPriceKey, Price, PriceProvider};
use std::collections::HashMap;
use std::time::Duration;

use crate::DatabaseClient;
use crate::database::assets::AssetsStore;
use crate::database::prices::{AssetsWithPricesFilter, PriceFilter, PricesStore};
use crate::database::prices_providers::PricesProvidersStore;
use crate::error::ResourceName;
use crate::models::{PriceAssetRow, PriceProviderConfigRow, PriceRow, price::NewPriceRow, price::PriceAssetDataRow};

pub const PRIMARY_PRICE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub trait PricesRepository {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, DatabaseError>;
    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, DatabaseError>;
    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, DatabaseError>;
    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceRow>, DatabaseError>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn get_prices_assets_by_provider(&mut self, provider: PriceProvider) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn get_primary_price_key(&mut self, asset_id: &str) -> Result<AssetPriceKey, DatabaseError>;
    fn get_primary_prices(&mut self, asset_ids: &[String]) -> Result<Vec<(AssetId, PriceRow)>, DatabaseError>;
    fn get_price_by_id(&mut self, price_id: &str) -> Result<Price, DatabaseError>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, DatabaseError>;
    fn get_assets_with_prices_by_filter(&mut self, filters: Vec<AssetsWithPricesFilter>) -> Result<Vec<PriceAssetDataRow>, DatabaseError>;
    fn get_assets_with_prices(&mut self, asset_ids: Vec<String>) -> Result<Vec<PriceAssetDataRow>, DatabaseError>;
}

impl PricesRepository for DatabaseClient {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::add_prices(self, values)?)
    }

    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::set_prices(self, values)?)
    }

    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::set_prices_assets(self, values)?)
    }

    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceRow>, DatabaseError> {
        Ok(PricesStore::get_prices_by_filter(self, filters)?)
    }

    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets(self)?)
    }

    fn get_prices_assets_by_provider(&mut self, provider: PriceProvider) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_by_provider(self, provider)?)
    }

    fn get_primary_price_key(&mut self, asset_id: &str) -> Result<AssetPriceKey, DatabaseError> {
        let providers = PricesProvidersStore::get_prices_providers(self)?;
        let rows = PricesStore::get_prices_for_asset_ids(self, &[asset_id.to_string()])?
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>();
        resolve_primary(&providers, &rows, PRIMARY_PRICE_MAX_AGE)
            .map(|row| AssetPriceKey::new(row.provider.0, row.provider_price_id.clone()))
            .ok_or_else(|| DatabaseError::not_found(PriceRow::RESOURCE_NAME, asset_id.to_string()))
    }

    fn get_primary_prices(&mut self, asset_ids: &[String]) -> Result<Vec<(AssetId, PriceRow)>, DatabaseError> {
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }
        let providers = PricesProvidersStore::get_prices_providers(self)?;
        let mut rows_by_asset: HashMap<String, Vec<PriceRow>> = PricesStore::get_prices_for_asset_ids(self, asset_ids)?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (id, row)| {
                acc.entry(id).or_default().push(row);
                acc
            });
        Ok(asset_ids
            .iter()
            .filter_map(|asset_id| {
                let rows = rows_by_asset.remove(asset_id)?;
                let row = resolve_primary(&providers, &rows, PRIMARY_PRICE_MAX_AGE)?.clone();
                let id = AssetId::new(asset_id)?;
                Some((id, row))
            })
            .collect())
    }

    fn get_price_by_id(&mut self, price_id: &str) -> Result<Price, DatabaseError> {
        Ok(PricesStore::get_price_by_id(self, price_id).or_not_found(price_id.to_string())?.as_primitive())
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_for_price_ids(self, ids)?)
    }

    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::delete_prices(self, ids)?)
    }

    fn get_assets_with_prices_by_filter(&mut self, filters: Vec<AssetsWithPricesFilter>) -> Result<Vec<PriceAssetDataRow>, DatabaseError> {
        let since = filters.iter().map(|AssetsWithPricesFilter::UpdatedSince(value)| *value).next();
        let asset_ids = match since {
            Some(value) => {
                let mut asset_ids = AssetsStore::get_asset_ids_updated_since(self, value)?;
                asset_ids.extend(PricesStore::get_asset_ids_updated_since(self, value)?);
                asset_ids.sort();
                asset_ids.dedup();
                asset_ids
            }
            None => AssetsStore::get_all_asset_ids(self)?,
        };
        self.get_assets_with_prices(asset_ids)
    }

    fn get_assets_with_prices(&mut self, asset_ids: Vec<String>) -> Result<Vec<PriceAssetDataRow>, DatabaseError> {
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let providers = PricesProvidersStore::get_prices_providers(self)?;
        let assets = AssetsStore::get_assets(self, asset_ids)?;
        let mut prices_by_asset: HashMap<String, Vec<PriceRow>> = PricesStore::get_prices_for_asset_ids(self, &assets.iter().map(|a| a.id.clone()).collect::<Vec<_>>())?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (asset_id, row)| {
                acc.entry(asset_id).or_default().push(row);
                acc
            });

        Ok(assets
            .into_iter()
            .map(|asset| {
                let rows = prices_by_asset.remove(&asset.id).unwrap_or_default();
                let price = resolve_primary(&providers, &rows, PRIMARY_PRICE_MAX_AGE).cloned();
                PriceAssetDataRow { asset, price }
            })
            .collect())
    }
}

fn resolve_primary<'a>(providers: &[PriceProviderConfigRow], rows: &'a [PriceRow], max_age: Duration) -> Option<&'a PriceRow> {
    let cutoff = (Utc::now() - chrono::Duration::from_std(max_age).ok()?).naive_utc();
    let mut candidates: Vec<(&PriceProviderConfigRow, &PriceRow)> = providers
        .iter()
        .filter(|p| p.enabled)
        .filter_map(|p| rows.iter().find(|row| row.provider.0 == p.id.0).map(|row| (p, row)))
        .filter(|(_, row)| row.last_updated_at >= cutoff)
        .collect();
    candidates.sort_by_key(|(p, _)| p.priority);
    candidates.first().map(|(_, row)| *row)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aged(provider: PriceProvider, seconds_ago: i64) -> PriceRow {
        let mut row = PriceRow::mock(provider, "x");
        row.last_updated_at = (Utc::now() - chrono::Duration::seconds(seconds_ago)).naive_utc();
        row
    }

    #[test]
    fn test_resolve_primary() {
        let providers = vec![
            PriceProviderConfigRow::new(PriceProvider::Coingecko, true),
            PriceProviderConfigRow::new(PriceProvider::Pyth, true),
            PriceProviderConfigRow::new(PriceProvider::Jupiter, false),
        ];
        let max_age = Duration::from_secs(3600);

        let fresh = vec![aged(PriceProvider::Coingecko, 60), aged(PriceProvider::Pyth, 60)];
        assert_eq!(resolve_primary(&providers, &fresh, max_age).unwrap().provider.0, PriceProvider::Coingecko);

        let stale_primary = vec![aged(PriceProvider::Coingecko, 7200), aged(PriceProvider::Pyth, 60)];
        assert_eq!(resolve_primary(&providers, &stale_primary, max_age).unwrap().provider.0, PriceProvider::Pyth);

        let only_disabled = vec![aged(PriceProvider::Jupiter, 60)];
        assert!(resolve_primary(&providers, &only_disabled, max_age).is_none());

        assert!(resolve_primary(&providers, &[], max_age).is_none());
    }
}
