use crate::schema::fiat_providers;
use crate::sql_types::{AssetId, FiatProviderNameRow};
use crate::{DatabaseClient, models::*};
use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::{FiatProviderName, FiatTransactionUpdate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FiatAssetFilter {
    HasAssetId,
    IsEnabled(bool),
    IsEnabledByProvider(bool),
    IsBuyEnabled(bool),
    IsSellEnabled(bool),
    ProviderEnabled(bool),
    ProviderBuyEnabled(bool),
    ProviderSellEnabled(bool),
}

pub(crate) trait FiatStore {
    fn add_fiat_assets(&mut self, values: Vec<FiatAssetRow>) -> Result<usize, diesel::result::Error>;
    fn add_fiat_providers(&mut self, values: Vec<FiatProviderRow>) -> Result<usize, diesel::result::Error>;
    fn add_fiat_providers_countries(&mut self, values: Vec<FiatProviderCountryRow>) -> Result<usize, diesel::result::Error>;
    fn get_fiat_providers_countries(&mut self) -> Result<Vec<FiatProviderCountryRow>, diesel::result::Error>;
    fn update_fiat_transaction(&mut self, provider: FiatProviderName, update: FiatTransactionUpdate) -> Result<FiatTransactionRow, diesel::result::Error>;
    fn get_fiat_transactions_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<FiatTransactionRow>, diesel::result::Error>;
    fn get_fiat_transactions_with_assets_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<(FiatTransactionRow, AssetRow)>, diesel::result::Error>;
    fn get_fiat_assets_by_filter(&mut self, filters: Vec<FiatAssetFilter>) -> Result<Vec<FiatAssetRow>, diesel::result::Error>;
    fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<AssetId>, diesel::result::Error>;
    fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<FiatAssetRow>, diesel::result::Error>;
    fn set_fiat_rates(&mut self, rates: Vec<FiatRateRow>) -> Result<usize, diesel::result::Error>;
    fn get_fiat_rates(&mut self) -> Result<Vec<FiatRateRow>, diesel::result::Error>;
    fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRateRow, diesel::result::Error>;
    fn get_fiat_providers(&mut self) -> Result<Vec<FiatProviderRow>, diesel::result::Error>;
    fn add_fiat_transaction(&mut self, transaction: NewFiatTransactionRow) -> Result<usize, diesel::result::Error>;
    fn update_fiat_provider_payment_methods(&mut self, provider_id: FiatProviderName, values: serde_json::Value) -> Result<usize, diesel::result::Error>;
}

impl FiatStore for DatabaseClient {
    fn add_fiat_assets(&mut self, values: Vec<FiatAssetRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        diesel::insert_into(fiat_assets)
            .values(values)
            .on_conflict(id)
            .do_update()
            .set((
                asset_id.eq(excluded(asset_id)),
                symbol.eq(excluded(symbol)),
                network.eq(excluded(network)),
                token_id.eq(excluded(token_id)),
                unsupported_countries.eq(excluded(unsupported_countries)),
                buy_limits.eq(excluded(buy_limits)),
                sell_limits.eq(excluded(sell_limits)),
                is_buy_enabled.eq(excluded(is_buy_enabled)),
                is_sell_enabled.eq(excluded(is_sell_enabled)),
                is_enabled_by_provider.eq(excluded(is_enabled_by_provider)),
            ))
            .execute(&mut self.connection)
    }

    fn add_fiat_providers(&mut self, values: Vec<FiatProviderRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_providers::dsl::*;
        diesel::insert_into(fiat_providers).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn add_fiat_providers_countries(&mut self, values: Vec<FiatProviderCountryRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_providers_countries::dsl::*;
        diesel::insert_into(fiat_providers_countries)
            .values(values)
            .on_conflict(id)
            .do_update()
            .set((alpha2.eq(excluded(alpha2)), is_allowed.eq(excluded(is_allowed))))
            .execute(&mut self.connection)
    }

    fn get_fiat_providers_countries(&mut self) -> Result<Vec<FiatProviderCountryRow>, diesel::result::Error> {
        use crate::schema::fiat_providers_countries::dsl::*;
        fiat_providers_countries.select(FiatProviderCountryRow::as_select()).load(&mut self.connection)
    }

    fn update_fiat_transaction(&mut self, provider: FiatProviderName, update: FiatTransactionUpdate) -> Result<FiatTransactionRow, diesel::result::Error> {
        use crate::schema::fiat_transactions::dsl::*;
        let provider = FiatProviderNameRow::from(provider);

        let patch = UpdateFiatTransactionRow::from_primitive(&update);
        if let Some(transaction) = diesel::update(
            fiat_transactions
                .filter(provider_id.eq(&provider))
                .filter(provider_transaction_id.eq(&update.transaction_id)),
        )
        .set(&patch)
        .returning(FiatTransactionRow::as_returning())
        .get_result(&mut self.connection)
        .optional()?
        {
            return Ok(transaction);
        }

        match update.provider_transaction_id.as_deref() {
            Some(provider_tx_id) => diesel::update(fiat_transactions.filter(provider_id.eq(&provider)).filter(quote_id.eq(&update.transaction_id)))
                .set((provider_transaction_id.eq(provider_tx_id), &patch))
                .returning(FiatTransactionRow::as_returning())
                .get_result(&mut self.connection),
            None => diesel::update(fiat_transactions.filter(provider_id.eq(&provider)).filter(quote_id.eq(&update.transaction_id)))
                .set(&patch)
                .returning(FiatTransactionRow::as_returning())
                .get_result(&mut self.connection),
        }
    }

    fn get_fiat_transactions_by_addresses(&mut self, addresses_list: Vec<String>) -> Result<Vec<FiatTransactionRow>, diesel::result::Error> {
        use crate::schema::fiat_transactions::dsl::*;

        if addresses_list.is_empty() {
            return Ok(vec![]);
        }

        fiat_transactions
            .filter(address.eq_any(addresses_list))
            .order(created_at.desc())
            .select(FiatTransactionRow::as_select())
            .load(&mut self.connection)
    }

    fn get_fiat_transactions_with_assets_by_addresses(&mut self, addresses_list: Vec<String>) -> Result<Vec<(FiatTransactionRow, AssetRow)>, diesel::result::Error> {
        use crate::schema::{assets, fiat_transactions};

        if addresses_list.is_empty() {
            return Ok(vec![]);
        }

        fiat_transactions::table
            .inner_join(assets::table)
            .filter(fiat_transactions::address.eq_any(addresses_list))
            .order(fiat_transactions::created_at.desc())
            .select((FiatTransactionRow::as_select(), AssetRow::as_select()))
            .load(&mut self.connection)
    }

    fn get_fiat_assets_by_filter(&mut self, filters: Vec<FiatAssetFilter>) -> Result<Vec<FiatAssetRow>, diesel::result::Error> {
        use crate::schema::{fiat_assets, fiat_providers};

        let mut query = fiat_assets::table.inner_join(fiat_providers::table).into_boxed();

        for filter in filters {
            query = match filter {
                FiatAssetFilter::HasAssetId => query.filter(fiat_assets::asset_id.is_not_null()),
                FiatAssetFilter::IsEnabled(value) => query.filter(fiat_assets::is_enabled.eq(value)),
                FiatAssetFilter::IsEnabledByProvider(value) => query.filter(fiat_assets::is_enabled_by_provider.eq(value)),
                FiatAssetFilter::IsBuyEnabled(value) => query.filter(fiat_assets::is_buy_enabled.eq(value)),
                FiatAssetFilter::IsSellEnabled(value) => query.filter(fiat_assets::is_sell_enabled.eq(value)),
                FiatAssetFilter::ProviderEnabled(value) => query.filter(fiat_providers::enabled.eq(value)),
                FiatAssetFilter::ProviderBuyEnabled(value) => query.filter(fiat_providers::buy_enabled.eq(value)),
                FiatAssetFilter::ProviderSellEnabled(value) => query.filter(fiat_providers::sell_enabled.eq(value)),
            };
        }

        query
            .select(FiatAssetRow::as_select())
            .distinct()
            .order(fiat_assets::asset_id.asc())
            .load(&mut self.connection)
    }

    fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<AssetId>, diesel::result::Error> {
        use crate::schema::fiat_transactions::dsl::*;

        fiat_transactions
            .filter(created_at.gt(from))
            .select(asset_id)
            .group_by(asset_id)
            .order_by(count_star().desc())
            .limit(limit)
            .load::<AssetId>(&mut self.connection)
    }

    fn get_fiat_assets_for_asset_id(&mut self, requested_asset_id: &str) -> Result<Vec<FiatAssetRow>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets::table()
            .inner_join(fiat_providers::table)
            .filter(fiat_providers::enabled.eq(true))
            .filter(asset_id.eq(requested_asset_id))
            .select(FiatAssetRow::as_select())
            .load(&mut self.connection)
    }

    fn set_fiat_rates(&mut self, rates: Vec<FiatRateRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        diesel::insert_into(fiat_rates)
            .values(&rates)
            .on_conflict(id)
            .do_update()
            .set(rate.eq(excluded(rate)))
            .execute(&mut self.connection)
    }

    fn get_fiat_rates(&mut self) -> Result<Vec<FiatRateRow>, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates.select(FiatRateRow::as_select()).load(&mut self.connection)
    }

    fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRateRow, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates.find(currency).select(FiatRateRow::as_select()).first(&mut self.connection)
    }

    fn get_fiat_providers(&mut self) -> Result<Vec<FiatProviderRow>, diesel::result::Error> {
        use crate::schema::fiat_providers::dsl::*;
        fiat_providers.select(FiatProviderRow::as_select()).load(&mut self.connection)
    }

    fn add_fiat_transaction(&mut self, transaction: NewFiatTransactionRow) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_transactions::dsl::*;
        diesel::insert_into(fiat_transactions)
            .values(&transaction)
            .on_conflict(quote_id)
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn update_fiat_provider_payment_methods(&mut self, provider_id_value: FiatProviderName, values: serde_json::Value) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_providers::dsl::*;
        diesel::update(fiat_providers.filter(id.eq(FiatProviderNameRow::from(provider_id_value))))
            .set(payment_methods.eq(values))
            .execute(&mut self.connection)
    }
}

// Public methods for backward compatibility
impl DatabaseClient {
    pub fn add_fiat_assets(&mut self, values: Vec<FiatAssetRow>) -> Result<usize, diesel::result::Error> {
        FiatStore::add_fiat_assets(self, values)
    }

    pub fn add_fiat_providers(&mut self, values: Vec<FiatProviderRow>) -> Result<usize, diesel::result::Error> {
        FiatStore::add_fiat_providers(self, values)
    }

    pub fn add_fiat_providers_countries(&mut self, values: Vec<FiatProviderCountryRow>) -> Result<usize, diesel::result::Error> {
        FiatStore::add_fiat_providers_countries(self, values)
    }

    pub fn get_fiat_providers_countries(&mut self) -> Result<Vec<FiatProviderCountryRow>, diesel::result::Error> {
        FiatStore::get_fiat_providers_countries(self)
    }

    pub fn update_fiat_transaction(&mut self, provider: FiatProviderName, update: FiatTransactionUpdate) -> Result<FiatTransactionRow, diesel::result::Error> {
        FiatStore::update_fiat_transaction(self, provider, update)
    }

    pub fn get_fiat_transactions_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<FiatTransactionRow>, diesel::result::Error> {
        FiatStore::get_fiat_transactions_by_addresses(self, addresses)
    }

    pub fn get_fiat_transactions_with_assets_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<(FiatTransactionRow, AssetRow)>, diesel::result::Error> {
        FiatStore::get_fiat_transactions_with_assets_by_addresses(self, addresses)
    }
    pub fn get_fiat_assets_by_filter(&mut self, filters: Vec<FiatAssetFilter>) -> Result<Vec<FiatAssetRow>, diesel::result::Error> {
        FiatStore::get_fiat_assets_by_filter(self, filters)
    }

    pub fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<AssetId>, diesel::result::Error> {
        FiatStore::get_fiat_assets_popular(self, from, limit)
    }

    pub fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<FiatAssetRow>, diesel::result::Error> {
        FiatStore::get_fiat_assets_for_asset_id(self, asset_id)
    }

    pub fn set_fiat_rates(&mut self, rates: Vec<FiatRateRow>) -> Result<usize, diesel::result::Error> {
        FiatStore::set_fiat_rates(self, rates)
    }

    pub fn get_fiat_rates(&mut self) -> Result<Vec<FiatRateRow>, diesel::result::Error> {
        FiatStore::get_fiat_rates(self)
    }

    pub fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRateRow, diesel::result::Error> {
        FiatStore::get_fiat_rate(self, currency)
    }

    pub fn get_fiat_providers(&mut self) -> Result<Vec<FiatProviderRow>, diesel::result::Error> {
        FiatStore::get_fiat_providers(self)
    }

    pub fn add_fiat_transaction(&mut self, transaction: NewFiatTransactionRow) -> Result<usize, diesel::result::Error> {
        FiatStore::add_fiat_transaction(self, transaction)
    }

    pub fn update_fiat_provider_payment_methods(&mut self, provider_id: FiatProviderName, values: serde_json::Value) -> Result<usize, diesel::result::Error> {
        FiatStore::update_fiat_provider_payment_methods(self, provider_id, values)
    }
}
