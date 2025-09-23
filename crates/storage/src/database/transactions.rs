use crate::{DatabaseClient, models::*, schema::transactions_addresses};
use chrono::DateTime;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::upsert::excluded;

pub(crate) trait TransactionsStore {
    fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, diesel::result::Error>;
    fn add_transactions(&mut self, transactions_values: Vec<Transaction>, addresses_values: Vec<TransactionAddresses>) -> Result<bool, diesel::result::Error>;
    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        options: primitives::TransactionsFetchOption,
    ) -> Result<Vec<Transaction>, diesel::result::Error>;
    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<crate::models::AddressChainIdResult>, diesel::result::Error>;
    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, diesel::result::Error>;
    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<String>, diesel::result::Error>;
    fn delete_transactions_by_ids(&mut self, ids: Vec<String>) -> Result<usize, diesel::result::Error>;
    fn add_transactions_types(&mut self, values: Vec<TransactionType>) -> Result<usize, diesel::result::Error>;
}

impl TransactionsStore for DatabaseClient {
    fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        transactions
            .filter(id.eq(_id))
            .order(created_at.asc())
            .select(Transaction::as_select())
            .load(&mut self.connection)
    }

    fn add_transactions(&mut self, transactions_values: Vec<Transaction>, addresses_values: Vec<TransactionAddresses>) -> Result<bool, diesel::result::Error> {
        self.connection
            .build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|conn: &mut diesel::pg::PgConnection| {
                use crate::schema::transactions::dsl::*;
                let query1 = diesel::insert_into(transactions)
                    .values(transactions_values)
                    .on_conflict(crate::schema::transactions::id)
                    .do_update()
                    .set((
                        from_address.eq(excluded(from_address)),
                        to_address.eq(excluded(to_address)),
                        value.eq(excluded(value)),
                        kind.eq(excluded(kind)),
                        state.eq(excluded(state)),
                        fee.eq(excluded(fee)),
                        fee_asset_id.eq(excluded(fee_asset_id)),
                        memo.eq(excluded(memo)),
                        metadata.eq(excluded(metadata)),
                        utxo_inputs.eq(excluded(utxo_inputs)),
                        utxo_outputs.eq(excluded(utxo_outputs)),
                    ))
                    .execute(conn);

                if let Some(error) = query1.err() {
                    return Err(error);
                }

                use crate::schema::transactions_addresses::dsl::*;
                let query2 = diesel::insert_into(transactions_addresses)
                    .values(&addresses_values)
                    .on_conflict((
                        crate::schema::transactions_addresses::transaction_id,
                        crate::schema::transactions_addresses::address,
                        crate::schema::transactions_addresses::asset_id,
                    ))
                    .do_nothing()
                    .execute(conn);

                if let Some(error) = query2.err() {
                    return Err(error);
                }

                Ok(true)
            })
    }

    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        options: primitives::TransactionsFetchOption,
    ) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;

        let mut query = transactions
            .into_boxed()
            .inner_join(transactions_addresses::table)
            .filter(transactions_addresses::chain_id.eq_any(chains.clone()))
            .filter(transactions_addresses::address.eq_any(addresses));

        if let Some(_asset_id) = options.asset_id {
            query = query.filter(asset_id.eq(_asset_id));
        }

        if let Some(from_timestamp) = options.from_timestamp {
            let datetime = DateTime::from_timestamp(from_timestamp.into(), 0).unwrap().naive_utc();
            query = query.filter(created_at.gt(datetime).or(updated_at.gt(datetime)));
        }

        query.order(created_at.desc()).select(Transaction::as_select()).load(&mut self.connection)
    }

    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResult>, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        transactions_addresses
            .select((address, chain_id))
            .group_by((address, chain_id))
            .having(count(address).gt(min_count))
            .order_by(count(address).desc())
            .limit(limit)
            .load::<AddressChainIdResult>(&mut self.connection)
    }

    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        diesel::delete(transactions_addresses)
            .filter(address.eq_any(addresses))
            .execute(&mut self.connection)
    }

    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        use crate::schema::transactions_addresses::dsl as addr;

        transactions
            .left_outer_join(addr::transactions_addresses.on(id.eq(addr::transaction_id)))
            .filter(addr::transaction_id.is_null())
            .select(id)
            .limit(limit)
            .load(&mut self.connection)
    }

    fn delete_transactions_by_ids(&mut self, ids: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        diesel::delete(transactions.filter(id.eq_any(ids))).execute(&mut self.connection)
    }

    fn add_transactions_types(&mut self, values: Vec<TransactionType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions_types::dsl::*;
        diesel::insert_into(transactions_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
