use std::collections::HashMap;

use diesel::prelude::*;
use primitives::{FiatAsset, FiatProvider, FiatProviderCountry, FiatProviderName, FiatQuote, FiatRate, FiatTransaction, fiat_assets::FiatAssetLimits};
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
    pub asset_id: Option<String>,
    pub provider: String,
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
    pub fn from_primitive(asset: FiatAsset) -> Self {
        let id = format!("{}_{}", asset.provider, asset.id).to_lowercase();
        Self {
            id,
            asset_id: asset.asset_id.map(|x| x.to_string()),
            provider: asset.provider,
            code: asset.id,
            symbol: asset.symbol,
            network: asset.network,
            token_id: asset.token_id,
            is_enabled: asset.enabled,
            is_enabled_by_provider: asset.enabled,
            is_buy_enabled: asset.is_buy_enabled,
            is_sell_enabled: asset.is_sell_enabled,
            buy_limits: Some(serde_json::to_value(asset.buy_limits).unwrap()),
            sell_limits: Some(serde_json::to_value(asset.sell_limits).unwrap()),
            unsupported_countries: Some(serde_json::to_value(asset.unsupported_countries).unwrap()),
        }
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
        self.unsupported_countries
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn buy_limits(&self) -> Vec<FiatAssetLimits> {
        self.buy_limits
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn sell_limits(&self) -> Vec<FiatAssetLimits> {
        self.sell_limits
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProviderRow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub buy_enabled: bool,
    pub sell_enabled: bool,
    pub priority: Option<i32>,
    pub priority_threshold_bps: Option<i32>,
}

impl FiatProviderRow {
    pub fn from_primitive(provider: FiatProviderName) -> Self {
        Self {
            id: provider.id(),
            name: provider.as_ref().to_string(),
            enabled: true,
            buy_enabled: true,
            sell_enabled: true,
            priority: None,
            priority_threshold_bps: None,
        }
    }

    pub fn as_primitive(&self) -> FiatProvider {
        FiatProvider {
            id: self.id.clone(),
            name: self.name.clone(),
            image_url: Some("".to_string()),
            priority: self.priority,
            threshold_bps: self.priority_threshold_bps,
            enabled: self.enabled,
            buy_enabled: self.buy_enabled,
            sell_enabled: self.sell_enabled,
        }
    }

    pub fn is_buy_enabled(&self) -> bool {
        self.enabled && self.buy_enabled
    }

    pub fn is_sell_enabled(&self) -> bool {
        self.enabled && self.sell_enabled
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatTransactionRow {
    pub asset_id: Option<String>,
    pub transaction_type: String,
    pub symbol: String,
    pub provider_id: String,
    pub provider_transaction_id: String,
    pub status: String,
    pub country: Option<String>,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub address: Option<String>,
    pub transaction_hash: Option<String>,
}

impl FiatTransactionRow {
    pub fn from_primitive(transaction: FiatTransaction) -> Self {
        Self {
            asset_id: transaction.asset_id.map(|x| x.to_string()),
            transaction_type: transaction.transaction_type.as_ref().to_string(),
            symbol: transaction.symbol,
            provider_id: transaction.provider_id,
            provider_transaction_id: transaction.provider_transaction_id,
            status: transaction.status.as_ref().to_string(),
            country: transaction.country,
            fiat_amount: transaction.fiat_amount,
            fiat_currency: transaction.fiat_currency,
            transaction_hash: transaction.transaction_hash,
            address: transaction.address,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_providers_countries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProviderCountryRow {
    pub id: String,
    pub provider: String,
    pub alpha2: String,
    pub is_allowed: bool,
}

impl FiatProviderCountryRow {
    pub fn from_primitive(primitive: FiatProviderCountry) -> Self {
        Self {
            id: format!("{}_{}", primitive.provider, primitive.alpha2).to_lowercase(),
            provider: primitive.provider.to_string(),
            alpha2: primitive.alpha2.to_string(),
            is_allowed: primitive.is_allowed,
        }
    }

    pub fn as_primitive(&self) -> FiatProviderCountry {
        FiatProviderCountry {
            provider: self.provider.clone(),
            alpha2: self.alpha2.clone(),
            is_allowed: self.is_allowed,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::fiat_transactions)]
pub struct FiatTransactionUpdateRow {
    pub status: String,
    pub country: Option<String>,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_quotes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatQuoteRow {
    pub id: String,
    pub provider_id: String,
    pub asset_id: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
}

impl FiatQuoteRow {
    pub fn from_primitive(quote: &FiatQuote) -> Self {
        Self {
            id: quote.id.clone(),
            provider_id: quote.provider.id.clone(),
            asset_id: quote.asset_id.clone(),
            fiat_amount: quote.fiat_amount,
            fiat_currency: quote.fiat_currency.clone(),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::fiat_quotes_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatQuoteRequestRow {
    pub device_id: i32,
    pub quote_id: String,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::fiat_webhooks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFiatWebhookRow {
    pub provider: String,
    pub transaction_id: Option<String>,
    pub payload: serde_json::Value,
    pub error: Option<String>,
}
