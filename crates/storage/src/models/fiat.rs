use std::collections::HashMap;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::fiat_rates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatRate {
    pub id: String,
    pub name: String,
    pub rate: f64,
}

impl FiatRate {
    pub fn as_primitive(&self) -> primitives::FiatRate {
        primitives::FiatRate {
            symbol: self.id.clone(),
            rate: self.rate,
        }
    }

    pub fn from_primitive(rate: primitives::FiatRate) -> Self {
        FiatRate {
            id: rate.symbol,
            name: "".to_string(),
            rate: rate.rate,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatAsset {
    pub id: String,
    pub asset_id: Option<String>,
    pub provider: String,
    pub code: String,
    pub symbol: String,
    pub network: Option<String>,
    pub token_id: Option<String>,
    pub is_enabled: bool, // managed by db
    pub is_enabled_by_provider: bool,
    pub buy_limits: Option<serde_json::Value>,
    pub sell_limits: Option<serde_json::Value>,
    pub unsupported_countries: Option<serde_json::Value>,
}

impl FiatAsset {
    pub fn from_primitive(asset: primitives::FiatAsset) -> Self {
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
            buy_limits: Some(serde_json::to_value(asset.buy_limits).unwrap()),
            sell_limits: Some(serde_json::to_value(asset.sell_limits).unwrap()),
            unsupported_countries: Some(serde_json::to_value(asset.unsupported_countries).unwrap()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled && self.is_enabled_by_provider
    }

    pub fn unsupported_countries(&self) -> HashMap<String, Vec<String>> {
        self.unsupported_countries
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn buy_limits(&self) -> Vec<primitives::fiat_assets::FiatAssetLimits> {
        self.buy_limits
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn sell_limits(&self) -> Vec<primitives::fiat_assets::FiatAssetLimits> {
        self.sell_limits
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProvider {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: Option<i32>,
    pub priority_threshold_bps: Option<i32>,
}

impl FiatProvider {
    pub fn from_primitive(provider: primitives::FiatProviderName) -> Self {
        Self {
            id: provider.id(),
            name: provider.as_ref().to_string(),
            enabled: true,
            priority: None,
            priority_threshold_bps: None,
        }
    }

    pub fn as_primitive(&self) -> primitives::FiatProvider {
        primitives::FiatProvider {
            id: self.id.clone(),
            name: self.name.clone(),
            image_url: Some("".to_string()),
            priority: self.priority,
            threshold_bps: self.priority_threshold_bps,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatTransaction {
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

impl FiatTransaction {
    pub fn from_primitive(transaction: primitives::FiatTransaction) -> Self {
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
pub struct FiatProviderCountry {
    pub id: String,
    pub provider: String,
    pub alpha2: String,
    pub is_allowed: bool,
}

impl FiatProviderCountry {
    pub fn from_primitive(primitive: primitives::FiatProviderCountry) -> Self {
        Self {
            id: format!("{}_{}", primitive.provider, primitive.alpha2).to_lowercase(),
            provider: primitive.provider.to_string(),
            alpha2: primitive.alpha2.to_string(),
            is_allowed: primitive.is_allowed,
        }
    }

    pub fn as_primitive(&self) -> primitives::FiatProviderCountry {
        primitives::FiatProviderCountry {
            provider: self.provider.clone(),
            alpha2: self.alpha2.clone(),
            is_allowed: self.is_allowed,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::fiat_transactions)]
pub struct FiatTransactionUpdate {
    pub status: String,
    pub country: Option<String>,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_quotes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatQuote {
    pub id: String,
    pub provider_id: String,
    pub asset_id: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
}

impl FiatQuote {
    pub fn from_primitive(quote: &primitives::FiatQuote) -> Self {
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
pub struct FiatQuoteRequest {
    pub device_id: String,
    pub quote_id: String,
}
