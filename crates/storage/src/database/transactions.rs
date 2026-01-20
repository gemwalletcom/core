use crate::{DatabaseClient, models::*, schema::transactions_addresses};
use chrono::DateTime;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::{Transaction, TransactionsFetchOption};

pub(crate) trait TransactionsStore {
    fn get_transaction_by_id(&mut self, chain: &str, hash: &str) -> Result<TransactionRow, diesel::result::Error>;
    fn add_transactions(&mut self, transactions: Vec<Transaction>) -> Result<usize, diesel::result::Error>;
    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        options: TransactionsFetchOption,
    ) -> Result<Vec<TransactionRow>, diesel::result::Error>;
    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResultRow>, diesel::result::Error>;
    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, diesel::result::Error>;
    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<i64>, diesel::result::Error>;
    fn delete_transactions_by_ids(&mut self, ids: Vec<i64>) -> Result<usize, diesel::result::Error>;
}

impl TransactionsStore for DatabaseClient {
    fn get_transaction_by_id(&mut self, chain: &str, hash: &str) -> Result<TransactionRow, diesel::result::Error> {
        use crate::schema::transactions::dsl;
        dsl::transactions
            .filter(dsl::chain.eq(chain))
            .filter(dsl::hash.eq(hash))
            .select(TransactionRow::as_select())
            .first(&mut self.connection)
    }

    fn add_transactions(&mut self, transactions: Vec<Transaction>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl;

        self.connection
            .build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|conn: &mut diesel::pg::PgConnection| {
                let mut total_addresses = 0usize;

                for transaction in transactions {
                    let new_transaction = NewTransactionRow::from_primitive(transaction.clone());

                    let inserted: TransactionRow = diesel::insert_into(dsl::transactions)
                        .values(&new_transaction)
                        .on_conflict((dsl::chain, dsl::hash))
                        .do_update()
                        .set((
                            dsl::from_address.eq(excluded(dsl::from_address)),
                            dsl::to_address.eq(excluded(dsl::to_address)),
                            dsl::value.eq(excluded(dsl::value)),
                            dsl::kind.eq(excluded(dsl::kind)),
                            dsl::state.eq(excluded(dsl::state)),
                            dsl::fee.eq(excluded(dsl::fee)),
                            dsl::fee_asset_id.eq(excluded(dsl::fee_asset_id)),
                            dsl::memo.eq(excluded(dsl::memo)),
                            dsl::metadata.eq(excluded(dsl::metadata)),
                            dsl::utxo_inputs.eq(excluded(dsl::utxo_inputs)),
                            dsl::utxo_outputs.eq(excluded(dsl::utxo_outputs)),
                        ))
                        .returning(TransactionRow::as_select())
                        .get_result(conn)?;

                    let addresses = NewTransactionAddressesRow::from_transaction(inserted.id, &transaction);

                    if !addresses.is_empty() {
                        use crate::schema::transactions_addresses::dsl as addr_dsl;
                        total_addresses += diesel::insert_into(addr_dsl::transactions_addresses)
                            .values(&addresses)
                            .on_conflict((addr_dsl::transaction_id, addr_dsl::address, addr_dsl::asset_id))
                            .do_nothing()
                            .execute(conn)?;
                    }
                }

                Ok(total_addresses)
            })
    }

    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        options: TransactionsFetchOption,
    ) -> Result<Vec<TransactionRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;

        let mut query = transactions
            .into_boxed()
            .inner_join(transactions_addresses::table)
            .filter(chain.eq_any(chains.clone()))
            .filter(transactions_addresses::address.eq_any(addresses));

        if let Some(_asset_id) = options.asset_id {
            query = query.filter(asset_id.eq(_asset_id));
        }

        if let Some(from_timestamp) = options.from_timestamp {
            let datetime = DateTime::from_timestamp(from_timestamp.into(), 0).unwrap().naive_utc();
            query = query.filter(created_at.gt(datetime).or(updated_at.gt(datetime)));
        }

        query.order(created_at.desc()).select(TransactionRow::as_select()).distinct().load(&mut self.connection)
    }

    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResultRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl as tx_dsl;
        use crate::schema::transactions_addresses::dsl::*;

        transactions_addresses
            .inner_join(tx_dsl::transactions)
            .select((address, tx_dsl::chain))
            .group_by((address, tx_dsl::chain))
            .having(count(address).gt(min_count))
            .order_by(count(address).desc())
            .limit(limit)
            .load::<AddressChainIdResultRow>(&mut self.connection)
    }

    fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        diesel::delete(transactions_addresses).filter(address.eq_any(addresses)).execute(&mut self.connection)
    }

    fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<i64>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        use crate::schema::transactions_addresses::dsl as addr;

        transactions
            .left_outer_join(addr::transactions_addresses.on(id.eq(addr::transaction_id)))
            .filter(addr::transaction_id.is_null())
            .select(id)
            .limit(limit)
            .load(&mut self.connection)
    }

    fn delete_transactions_by_ids(&mut self, ids: Vec<i64>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        diesel::delete(transactions.filter(id.eq_any(ids))).execute(&mut self.connection)
    }
}
