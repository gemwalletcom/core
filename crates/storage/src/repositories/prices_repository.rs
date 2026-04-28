use crate::{DatabaseError, DieselResultExt};
use chrono::Utc;
use primitives::{AssetId, AssetIdVecExt, Price, PriceId, PriceProvider};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::DatabaseClient;
use crate::database::assets::AssetsStore;
use crate::database::charts::ChartsStore;
use crate::database::prices::{AssetsWithPricesFilter, PriceFilter, PriceUpdate, PricesStore};
use crate::database::prices_providers::PricesProvidersStore;
use crate::error::ResourceName;
use crate::models::min_max::MinMax;
use crate::models::{ChartRow, PriceAssetRow, PriceProviderConfigRow, PriceRow, price::NewPriceRow, price::PriceAssetDataRow, price::PricesChangeset};

pub trait PricesRepository {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, DatabaseError>;
    fn set_prices(&mut self, prices: Vec<PriceRow>) -> Result<Vec<AssetId>, DatabaseError>;
    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, DatabaseError>;
    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceRow>, DatabaseError>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn get_prices_assets_by_provider(&mut self, provider: PriceProvider) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn get_primary_price_key(&mut self, asset_id: &AssetId, max_age: Duration) -> Result<PriceId, DatabaseError>;
    fn get_primary_prices(&mut self, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<(AssetId, PriceRow)>, DatabaseError>;
    fn get_price_by_id(&mut self, price_id: &str) -> Result<Price, DatabaseError>;
    fn get_prices_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<PriceRow>, DatabaseError>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, DatabaseError>;
    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, DatabaseError>;
    fn get_assets_with_prices_by_filter(&mut self, filters: Vec<AssetsWithPricesFilter>, max_age: Duration) -> Result<Vec<PriceAssetDataRow>, DatabaseError>;
    fn get_assets_with_prices(&mut self, asset_ids: Vec<AssetId>, max_age: Duration) -> Result<Vec<PriceAssetDataRow>, DatabaseError>;
    fn update_prices(&mut self, price_ids: Vec<String>, updates: Vec<PriceUpdate>) -> Result<usize, DatabaseError>;
    fn update_extremes_for_price(&mut self, price_id: &str) -> Result<usize, DatabaseError>;
}

impl PricesRepository for DatabaseClient {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::add_prices(self, values)?)
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

    fn get_primary_price_key(&mut self, asset_id: &AssetId, max_age: Duration) -> Result<PriceId, DatabaseError> {
        let providers = PricesProvidersStore::get_prices_providers(self)?;
        let rows = PricesStore::get_prices_for_asset_ids(self, &[asset_id.to_string()])?
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>();
        resolve_primary(&providers, &rows, max_age)
            .map(|row| PriceId::new(row.provider.0, row.provider_price_id.clone()))
            .ok_or_else(|| DatabaseError::not_found(PriceRow::RESOURCE_NAME, asset_id.to_string()))
    }

    fn get_primary_prices(&mut self, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<(AssetId, PriceRow)>, DatabaseError> {
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }
        let providers = PricesProvidersStore::get_prices_providers(self)?;
        let string_ids: Vec<String> = asset_ids.iter().map(|id| id.to_string()).collect();
        let mut rows_by_asset: HashMap<String, Vec<PriceRow>> = PricesStore::get_prices_for_asset_ids(self, &string_ids)?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (id, row)| {
                acc.entry(id).or_default().push(row);
                acc
            });
        Ok(asset_ids
            .iter()
            .filter_map(|asset_id| {
                let rows = rows_by_asset.remove(&asset_id.to_string())?;
                let row = resolve_primary(&providers, &rows, max_age)?.clone();
                Some((asset_id.clone(), row))
            })
            .collect())
    }

    fn get_price_by_id(&mut self, price_id: &str) -> Result<Price, DatabaseError> {
        Ok(PricesStore::get_price_by_id(self, price_id).or_not_found(price_id.to_string())?.as_primitive())
    }

    fn get_prices_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<PriceRow>, DatabaseError> {
        Ok(PricesStore::get_prices_for_asset_ids(self, &[asset_id.to_string()])?
            .into_iter()
            .map(|(_, row)| row)
            .collect())
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, DatabaseError> {
        Ok(PricesStore::get_prices_assets_for_price_ids(self, ids)?)
    }

    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(PricesStore::delete_prices(self, ids)?)
    }

    fn get_assets_with_prices_by_filter(&mut self, filters: Vec<AssetsWithPricesFilter>, max_age: Duration) -> Result<Vec<PriceAssetDataRow>, DatabaseError> {
        let since = filters.iter().map(|AssetsWithPricesFilter::UpdatedSince(value)| *value).next();
        let asset_ids: Vec<String> = match since {
            Some(value) => {
                let mut asset_ids = AssetsStore::get_asset_ids_updated_since(self, value)?;
                asset_ids.extend(PricesStore::get_asset_ids_updated_since(self, value)?);
                asset_ids.sort();
                asset_ids.dedup();
                asset_ids
            }
            None => AssetsStore::get_all_asset_ids(self)?,
        };
        let asset_ids = asset_ids.into_iter().filter_map(|id| AssetId::new(&id)).collect();
        self.get_assets_with_prices(asset_ids, max_age)
    }

    fn update_prices(&mut self, price_ids: Vec<String>, updates: Vec<PriceUpdate>) -> Result<usize, DatabaseError> {
        if updates.is_empty() {
            return Ok(0);
        }
        let changeset = PricesChangeset::from_updates(updates);
        Ok(PricesStore::update_prices(self, &price_ids, &changeset)?)
    }

    fn update_extremes_for_price(&mut self, price_id: &str) -> Result<usize, DatabaseError> {
        use primitives::ChartTimeframe;
        let row = PricesStore::get_price_by_id(self, price_id).or_not_found(price_id.to_string())?;
        let timeframes = [ChartTimeframe::Raw, ChartTimeframe::Hourly, ChartTimeframe::Daily];
        let extremes: Vec<MinMax<f64>> = timeframes
            .into_iter()
            .map(|tf| ChartsStore::get_chart_extremes(self, price_id, tf))
            .collect::<Result<_, _>>()?;
        let combined = MinMax {
            max: extremes.iter().filter_map(|e| e.max).max_by(|a, b| a.value.total_cmp(&b.value)),
            min: extremes.iter().filter_map(|e| e.min).min_by(|a, b| a.value.total_cmp(&b.value)),
        };
        let updates = row.merge_extremes_from_charts(combined);
        PricesRepository::update_prices(self, vec![price_id.to_string()], updates)
    }

    fn set_prices(&mut self, prices: Vec<PriceRow>) -> Result<Vec<AssetId>, DatabaseError> {
        if prices.is_empty() {
            return Ok(vec![]);
        }
        let price_ids: Vec<String> = prices.iter().map(|p| p.id.clone()).collect();
        let mappings = PricesStore::get_prices_assets_for_price_ids(self, price_ids)?;
        let mapped_ids: HashSet<String> = mappings.iter().map(|m| m.price_id.clone()).collect();
        let to_store: Vec<PriceRow> = prices.into_iter().filter(|p| mapped_ids.contains(&p.id)).collect();
        if to_store.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<String> = to_store.iter().map(|p| p.id.clone()).collect();
        let incoming_by_id: HashMap<String, PriceRow> = to_store.iter().cloned().map(|p| (p.id.clone(), p)).collect();
        PricesStore::set_prices(self, to_store)?;

        let current_prices = PricesStore::get_prices_by_filter(self, vec![PriceFilter::Ids(ids)])?;
        let extreme_updates: Vec<(String, Vec<PriceUpdate>)> = current_prices
            .iter()
            .filter_map(|price| {
                let updates = price.merge_extremes(incoming_by_id.get(&price.id));
                (!updates.is_empty()).then_some((price.id.clone(), updates))
            })
            .collect();
        for (id, updates) in extreme_updates {
            PricesRepository::update_prices(self, vec![id], updates)?;
        }

        let charts: Vec<ChartRow> = current_prices.iter().cloned().map(ChartRow::from_price).collect();
        ChartsStore::add_charts(self, primitives::ChartTimeframe::Raw, charts)?;

        Ok(mappings.into_iter().map(|m| m.asset_id.0).collect::<HashSet<_>>().into_iter().collect())
    }

    fn get_assets_with_prices(&mut self, asset_ids: Vec<AssetId>, max_age: Duration) -> Result<Vec<PriceAssetDataRow>, DatabaseError> {
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let providers = PricesProvidersStore::get_prices_providers(self)?;
        let assets = AssetsStore::get_assets(self, asset_ids.ids())?;
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
                let price = resolve_primary(&providers, &rows, max_age).cloned();
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
