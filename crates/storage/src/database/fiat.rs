use crate::schema::fiat_providers;
use crate::{models::*, DatabaseClient};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::upsert::excluded;

impl DatabaseClient {
    pub fn add_fiat_assets(&mut self, values: Vec<FiatAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        diesel::insert_into(fiat_assets)
            .values(values)
            .on_conflict((id,))
            .do_update()
            .set((
                asset_id.eq(excluded(asset_id)),
                symbol.eq(excluded(symbol)),
                network.eq(excluded(network)),
                token_id.eq(excluded(token_id)),
                enabled_by_provider.eq(excluded(enabled_by_provider)),
            ))
            .execute(&mut self.connection)
    }

    pub fn add_fiat_providers(&mut self, values: Vec<FiatProvider>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_providers::dsl::*;
        diesel::insert_into(fiat_providers)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_fiat_transaction(&mut self, transaction: FiatTransaction) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_transactions::dsl::*;

        let update = FiatTransactionUpdate {
            status: transaction.status.clone(),
            transaction_hash: transaction.transaction_hash.clone(),
            address: transaction.address.clone(),
            fee_provider: transaction.fee_provider,
            fee_partner: transaction.fee_partner,
            fee_network: transaction.fee_network,
        };

        diesel::insert_into(fiat_transactions)
            .values(&transaction)
            .on_conflict((provider_id, provider_transaction_id))
            .do_update()
            .set(update)
            .execute(&mut self.connection)
    }

    pub fn get_fiat_assets(&mut self) -> Result<Vec<FiatAsset>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets.select(FiatAsset::as_select()).load(&mut self.connection)
    }

    pub fn get_fiat_assets_is_buyable(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_buyable.eq(true)).select(id).load(&mut self.connection)
    }

    pub fn get_fiat_assets_is_sellable(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_sellable.eq(true)).select(id).load(&mut self.connection)
    }

    pub fn get_fiat_assets_for_asset_id(&mut self, _asset_id: &str) -> Result<Vec<FiatAsset>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets::table()
            .inner_join(fiat_providers::table)
            .filter(fiat_providers::enabled.eq(true))
            .filter(asset_id.eq(_asset_id))
            .select(FiatAsset::as_select())
            .load(&mut self.connection)
    }

    pub fn set_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        diesel::insert_into(fiat_rates)
            .values(&rates)
            .on_conflict(symbol)
            .do_update()
            .set(rate.eq(excluded(rate)))
            .execute(&mut self.connection)
    }

    pub fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates.select(FiatRate::as_select()).load(&mut self.connection)
    }

    pub fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRate, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates.filter(symbol.eq(currency)).select(FiatRate::as_select()).first(&mut self.connection)
    }
}
