pub mod assets;
pub mod assets_addresses;
pub mod assets_links;
pub mod assets_types;
pub mod device;
pub mod fiat;
pub mod link;
pub mod nft;
pub mod nodes;
pub mod parser_state;
pub mod price;
pub mod price_alerts;
pub mod release;
pub mod scan;
pub mod subscription;
pub mod tag;

use crate::models::*;
use crate::schema::transactions_addresses;
use chrono::DateTime;
use diesel::associations::HasTable;
use diesel::dsl::count;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{upsert::excluded, Connection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");

use primitives::TransactionsFetchOption;

pub struct DatabaseClient {
    pub(crate) connection: PgConnection,
}

impl DatabaseClient {
    pub fn new(database_url: &str) -> Self {
        let connection = PgConnection::establish(database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self { connection }
    }

    pub fn add_transactions(
        &mut self,
        transactions_values: Vec<Transaction>,
        addresses_values: Vec<TransactionAddresses>,
    ) -> Result<bool, diesel::result::Error> {
        self.connection
            .build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|conn: &mut PgConnection| {
                use crate::schema::transactions::dsl::*;
                let query1 = diesel::insert_into(transactions::table())
                    .values(transactions_values)
                    .on_conflict(super::schema::transactions::id)
                    .do_update()
                    .set((
                        block_number.eq(excluded(block_number)),
                        sequence.eq(excluded(sequence)),
                        fee.eq(excluded(fee)),
                        fee_asset_id.eq(excluded(fee_asset_id)),
                        memo.eq(excluded(memo)),
                        updated_at.eq(excluded(updated_at)),
                    ))
                    .execute(conn);

                if let Some(error) = query1.err() {
                    return Err(error);
                }

                use crate::schema::transactions_addresses::dsl::*;
                let query2 = diesel::insert_into(transactions_addresses::table())
                    .values(&addresses_values)
                    .on_conflict((
                        super::schema::transactions_addresses::transaction_id,
                        super::schema::transactions_addresses::address,
                        super::schema::transactions_addresses::asset_id,
                    ))
                    .do_nothing()
                    .execute(conn);

                if let Some(error) = query2.err() {
                    return Err(error);
                }

                Ok(true)
            })
    }

    pub fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        options: TransactionsFetchOption,
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
            query = query.filter(created_at.gt(datetime));
        }

        query.order(created_at.desc()).select(Transaction::as_select()).load(&mut self.connection)
    }

    pub fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        transactions
            .filter(id.eq(_id))
            .order(created_at.asc())
            .select(Transaction::as_select())
            .load(&mut self.connection)
    }

    pub fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResult>, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        transactions_addresses
            .select((address, chain_id))
            .group_by((address, chain_id))
            .having(count(address).gt(min_count))
            .order_by(count(address).desc())
            .limit(limit)
            .load::<AddressChainIdResult>(&mut self.connection)
    }

    pub fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        diesel::delete(transactions_addresses)
            .filter(address.eq_any(addresses))
            .execute(&mut self.connection)
    }

    pub fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        use crate::schema::transactions_addresses::dsl as addr;

        transactions
            .left_outer_join(addr::transactions_addresses.on(id.eq(addr::transaction_id)))
            .filter(addr::transaction_id.is_null())
            .select(id)
            .limit(limit)
            .load(&mut self.connection)
    }

    pub fn delete_transactions_by_ids(&mut self, ids: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        diesel::delete(transactions.filter(id.eq_any(ids))).execute(&mut self.connection)
    }

    pub fn get_asset(&mut self, asset_id: &str) -> Result<Asset, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(id.eq(asset_id)).select(Asset::as_select()).first(&mut self.connection)
    }

    pub fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(id.eq_any(asset_ids)).select(Asset::as_select()).load(&mut self.connection)
    }

    // swap

    pub fn get_swap_assets(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets
            .filter(rank.gt(21))
            .filter(is_swappable.eq(true))
            .select(id)
            .order(rank.desc())
            .load(&mut self.connection)
    }

    pub fn get_swap_assets_version(&mut self) -> Result<i32, diesel::result::Error> {
        Ok((std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() / 3600) as i32)
    }

    pub fn add_chains(&mut self, _chains: Vec<String>) -> Result<usize, diesel::result::Error> {
        let values = _chains.iter().map(|chain| super::models::Chain { id: chain.clone() }).collect::<Vec<_>>();

        use crate::schema::chains::dsl::*;
        diesel::insert_into(chains)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    // nft

    pub fn migrations(&mut self) {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
    }

    pub fn add_transactions_types(&mut self, values: Vec<TransactionType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions_types::dsl::*;
        diesel::insert_into(transactions_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
