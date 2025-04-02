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
    pub fn from_primitive(rate: primitives::FiatRate) -> Self {
        FiatRate {
            id: rate.symbol,
            name: rate.name,
            rate: rate.rate,
        }
    }

    pub fn multiplier(&self, base: f64) -> f64 {
        self.rate * base
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
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled && self.is_enabled_by_provider
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::fiat_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatProvider {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}

impl FiatProvider {
    pub fn from_primitive(provider: primitives::FiatProviderName) -> Self {
        Self {
            id: provider.id(),
            name: provider.as_ref().to_string(),
            enabled: true,
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
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub address: Option<String>,
    pub transaction_hash: Option<String>,
    pub fee_network: Option<f64>,
    pub fee_partner: Option<f64>,
    pub fee_provider: Option<f64>,
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
            fiat_amount: transaction.fiat_amount,
            fiat_currency: transaction.fiat_currency,
            transaction_hash: transaction.transaction_hash,
            address: transaction.address,
            fee_provider: transaction.fee_provider,
            fee_network: transaction.fee_network,
            fee_partner: transaction.fee_partner,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::fiat_transactions)]
pub struct FiatTransactionUpdate {
    pub status: String,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
    pub fee_network: Option<f64>,
    pub fee_partner: Option<f64>,
    pub fee_provider: Option<f64>,
}
