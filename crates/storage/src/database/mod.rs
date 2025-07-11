pub mod assets;
pub mod assets_addresses;
pub mod assets_links;
pub mod assets_types;

pub mod charts;
pub mod devices;
pub mod fiat;
pub mod link_types;
pub mod nft;
pub mod nodes;
pub mod parser_state;
pub mod price_alerts;
pub mod prices;
pub mod releases;
pub mod scan_addresses;
pub mod subscriptions;
pub mod tag;
pub mod transactions;
pub mod migrations;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");

pub struct DatabaseClient {
    connection: PgConnection,
}

impl DatabaseClient {
    pub fn new(database_url: &str) -> Self {
        let connection = PgConnection::establish(database_url).unwrap_or_else(|_| panic!("Error connecting to {database_url}"));

        Self { connection }
    }



}
