use std::collections::{HashMap, HashSet};

use crate::DatabaseError;
use crate::sql_types::{AssetId, FiatProviderNameRow, FiatTransactionStatusRow, FiatTransactionType};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{
    AssetId as PrimitiveAssetId, FiatAsset, FiatProvider, FiatProviderCountry, FiatProviderName, FiatRate, FiatTransaction, FiatTransactionUpdate, PaymentType,
    fiat_assets::FiatAssetLimits,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::fiat_rates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatRateRow {
    pub id: String,
    pub name: String,
    pub rate: f64,
}

impl FiatRateRow {
    pub fn as_primitive(&self) -> FiatRate {
        FiatRate {
            symbol: self.id.clone(),
            rate: self.rate,
        }
    }

    pub fn from_primitive(rate: FiatRate) -> Self {
        FiatRateRow {
            id: rate.symbol,
            name: "".to_string(),
            rate: rate.rate,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatAssetRow {
    pub id: String,
    pub asset_id: Option<AssetId>,
    pub provider: FiatProviderNameRow,
    pub code: String,
    pub symbol: String,
    pub network: Option<String>,
    pub token_id: Option<String>,
    pub is_enabled: bool, // managed by db
    pub is_enabled_by_provider: bool,
    pub is_buy_enabled: bool,
    pub is_sell_enabled: bool,
    pub buy_limits: Option<serde_json::Value>,
    pub sell_limits: Option<serde_json::Value>,
    pub unsupported_countries: Option<serde_json::Value>,
}

impl FiatAssetRow {
    pub fn from_primitive(asset: FiatAsset) -> Result<Self, DatabaseError> {
        let provider = FiatProviderNameRow::from(asset.provider);
        let id = format!("{}_{}", provider.0.id(), asset.id).to_lowercase();
        let buy_limits = Some(serde_json::to_value(asset.buy_limits)?);
        let sell_limits = Some(serde_json::to_value(asset.sell_limits)?);
        let unsupported_countries = Some(serde_json::to_value(asset.unsupported_countries)?);

        Ok(Self {
            id,
            asset_id: asset.asset_id.map(AssetId::from),
            provider,
            code: asset.id,
            symbol: asset.symbol,
            network: asset.network,
            token_id: asset.token_id,
            is_enabled: asset.enabled,
            is_enabled_by_provider: asset.enabled,
            is_buy_enabled: asset.is_buy_enabled,
            is_sell_enabled: asset.is_sell_enabled,
            buy_limits,
            sell_limits,
            unsupported_countries,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled && self.is_enabled_by_provider
    }

    pub fn is_buy_enabled(&self) -> bool {
        self.is_enabled() && self.is_buy_enabled
    }

    pub fn is_sell_enabled(&self) -> bool {
        self.is_enabled() && self.is_sell_enabled
    }

    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        self.unsupported_countries.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
    }

    pub fn buy_limits(&self) -> Vec<FiatAssetLimits> {
        self.buy_limits.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
    }

    pub fn sell_limits(&self) -> Vec<FiatAssetLimits> {
        self.sell_limits.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
    }
}

pub trait FiatAssetRowsExt {
    fn asset_ids(self) -> Vec<PrimitiveAssetId>;
}

impl FiatAssetRowsExt for Vec<FiatAssetRow> {
    fn asset_ids(self) -> Vec<PrimitiveAssetId> {
        self.into_iter()
            .filter_map(|x| x.asset_id.map(|asset_id| asset_id.0))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProviderRow {
    pub id: FiatProviderNameRow,
    pub name: String,
    pub enabled: bool,
    pub buy_enabled: bool,
    pub sell_enabled: bool,
    pub priority: Option<i32>,
    pub priority_threshold_bps: Option<i32>,
    pub payment_methods: serde_json::Value,
}

impl FiatProviderRow {
    pub fn from_primitive(provider: FiatProviderName) -> Self {
        Self {
            id: provider.into(),
            name: provider.name().to_string(),
            enabled: true,
            buy_enabled: true,
            sell_enabled: true,
            priority: None,
            priority_threshold_bps: None,
            payment_methods: serde_json::to_value(Vec::<PaymentType>::new()).unwrap(),
        }
    }

    pub fn as_primitive(&self) -> FiatProvider {
        let provider = self.id.0;
        let payment_methods: Vec<PaymentType> = serde_json::from_value(self.payment_methods.clone()).unwrap_or_default();

        FiatProvider {
            id: provider,
            name: provider.name().to_string(),
            image_url: None,
            priority: self.priority,
            threshold_bps: self.priority_threshold_bps,
            enabled: self.enabled,
            buy_enabled: self.buy_enabled,
            sell_enabled: self.sell_enabled,
            payment_methods,
        }
    }

    pub fn is_buy_enabled(&self) -> bool {
        self.enabled && self.buy_enabled
    }

    pub fn is_sell_enabled(&self) -> bool {
        self.enabled && self.sell_enabled
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::fiat_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatTransactionRow {
    pub id: i32,
    pub asset_id: AssetId,
    pub transaction_type: FiatTransactionType,
    pub provider_id: FiatProviderNameRow,
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatusRow,
    pub country: Option<String>,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub value: Option<String>,
    pub address_id: i32,
    pub transaction_hash: Option<String>,
    pub device_id: i32,
    pub wallet_id: i32,
    pub quote_id: String,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl FiatTransactionRow {
    pub fn as_primitive(&self) -> Result<FiatTransaction, DatabaseError> {
        let value = self
            .value
            .clone()
            .ok_or_else(|| DatabaseError::Error(format!("Fiat transaction {} is missing value", self.quote_id)))?;

        Ok(FiatTransaction {
            id: self.quote_id.clone(),
            asset_id: self.asset_id.0.clone(),
            transaction_type: self.transaction_type.0.clone(),
            provider: self.provider_id.0,
            provider_transaction_id: self.provider_transaction_id.clone(),
            status: self.status.0.clone(),
            country: self.country.clone(),
            fiat_amount: self.fiat_amount,
            fiat_currency: self.fiat_currency.clone(),
            value,
            transaction_hash: self.transaction_hash.clone(),
            created_at: self.created_at.and_utc(),
            updated_at: self.updated_at.and_utc(),
        })
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::fiat_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFiatTransactionRow {
    pub asset_id: AssetId,
    pub transaction_type: FiatTransactionType,
    pub provider_id: FiatProviderNameRow,
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatusRow,
    pub country: Option<String>,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub value: Option<String>,
    pub address_id: i32,
    pub transaction_hash: Option<String>,
    pub device_id: i32,
    pub wallet_id: i32,
    pub quote_id: String,
}

impl NewFiatTransactionRow {
    pub fn new(transaction: FiatTransaction, device_id: i32, wallet_id: i32, address_id: i32) -> Self {
        Self {
            asset_id: transaction.asset_id.into(),
            transaction_type: transaction.transaction_type.into(),
            provider_id: transaction.provider.into(),
            provider_transaction_id: transaction.provider_transaction_id,
            status: transaction.status.into(),
            country: transaction.country,
            fiat_amount: transaction.fiat_amount,
            fiat_currency: transaction.fiat_currency,
            value: Some(transaction.value),
            address_id,
            transaction_hash: transaction.transaction_hash,
            device_id,
            wallet_id,
            quote_id: transaction.id,
        }
    }

    pub fn from_existing(existing: &FiatTransactionRow, update: &FiatTransactionUpdate, provider_transaction_id: String) -> Self {
        Self {
            asset_id: existing.asset_id.clone(),
            transaction_type: existing.transaction_type.clone(),
            provider_id: existing.provider_id.clone(),
            provider_transaction_id: Some(provider_transaction_id),
            status: update.status.clone().into(),
            country: existing.country.clone(),
            fiat_amount: update.fiat_amount.unwrap_or(existing.fiat_amount),
            fiat_currency: update.fiat_currency.clone().unwrap_or_else(|| existing.fiat_currency.clone()),
            value: existing.value.clone(),
            address_id: existing.address_id,
            transaction_hash: update.transaction_hash.clone(),
            device_id: existing.device_id,
            wallet_id: existing.wallet_id,
            quote_id: existing.quote_id.clone(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_providers_countries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProviderCountryRow {
    pub id: String,
    pub provider: FiatProviderNameRow,
    pub alpha2: String,
    pub is_allowed: bool,
}

impl FiatProviderCountryRow {
    pub fn from_primitive(primitive: FiatProviderCountry) -> Self {
        let provider = FiatProviderNameRow::from(primitive.provider);

        Self {
            id: format!("{}_{}", provider.0.id(), primitive.alpha2).to_lowercase(),
            provider,
            alpha2: primitive.alpha2.to_string(),
            is_allowed: primitive.is_allowed,
        }
    }

    pub fn as_primitive(&self) -> FiatProviderCountry {
        FiatProviderCountry {
            provider: self.provider.0,
            alpha2: self.alpha2.clone(),
            is_allowed: self.is_allowed,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::fiat_transactions)]
pub struct UpdateFiatTransactionRow {
    pub status: FiatTransactionStatusRow,
    pub fiat_amount: Option<f64>,
    pub fiat_currency: Option<String>,
    pub transaction_hash: Option<String>,
}

impl UpdateFiatTransactionRow {
    pub fn from_primitive(transaction: &FiatTransactionUpdate) -> Self {
        Self {
            status: transaction.status.clone().into(),
            fiat_amount: transaction.fiat_amount,
            fiat_currency: transaction.fiat_currency.clone(),
            transaction_hash: transaction.transaction_hash.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FiatTransactionRow, UpdateFiatTransactionRow};
    use chrono::{DateTime, Utc};
    use primitives::{FiatTransactionStatus, FiatTransactionUpdate};

    #[test]
    fn as_primitive_maps_value_and_timestamps() {
        let row = FiatTransactionRow::mock_with_timestamps(DateTime::<Utc>::from_timestamp(1, 0).unwrap(), DateTime::<Utc>::from_timestamp(2, 0).unwrap());

        let transaction = row.as_primitive().unwrap();

        assert_eq!(transaction.id, "quote_123");
        assert_eq!(transaction.value, "123000000000000000");
        assert_eq!(transaction.created_at, DateTime::<Utc>::from_timestamp(1, 0).unwrap());
        assert_eq!(transaction.updated_at, DateTime::<Utc>::from_timestamp(2, 0).unwrap());
    }

    #[test]
    fn as_primitive_returns_error_without_value() {
        let row = FiatTransactionRow::mock_without_value();

        assert!(row.as_primitive().is_err());
    }

    #[test]
    fn update_row_maps_fiat_currency() {
        let update = FiatTransactionUpdate {
            transaction_id: "quote_123".to_string(),
            provider_transaction_id: Some("tx_123".to_string()),
            status: FiatTransactionStatus::Pending,
            transaction_hash: Some("0xabc".to_string()),
            fiat_amount: Some(100.0),
            fiat_currency: Some("EUR".to_string()),
        };

        let row = UpdateFiatTransactionRow::from_primitive(&update);

        assert_eq!(row.fiat_currency, Some("EUR".to_string()));
        assert_eq!(row.fiat_amount, Some(100.0));
    }
}
