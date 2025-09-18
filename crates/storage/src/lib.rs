use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

pub mod database;
pub mod error;
pub use self::database::{
    assets::{AssetFilter, AssetUpdate},
    DatabaseClient,
};
pub use self::error::DatabaseError;
pub mod models;
pub use self::models::ScanAddressType;
pub mod repositories;
pub use self::repositories::{
    assets_addresses_repository::AssetsAddressesRepository, assets_links_repository::AssetsLinksRepository, assets_repository::AssetsRepository,
    assets_types_repository::AssetsTypesRepository, charts_repository::ChartsRepository, devices_repository::DevicesRepository,
    fiat_repository::FiatRepository, link_types_repository::LinkTypesRepository, migrations_repository::MigrationsRepository, nft_repository::NftRepository,
    nodes_repository::NodesRepository, parser_state_repository::ParserStateRepository, price_alerts_repository::PriceAlertsRepository,
    prices_repository::PricesRepository, releases_repository::ReleasesRepository, scan_addresses_repository::ScanAddressesRepository,
    subscriptions_repository::SubscriptionsRepository, support_repository::SupportRepository, tag_repository::TagRepository,
    transactions_repository::TransactionsRepository,
};
pub mod schema;

pub struct RedisClient {
    conn: MultiplexedConnection,
}

impl RedisClient {
    pub async fn new(database_url: &str) -> RedisResult<Self> {
        let client = redis::Client::open(database_url)?;
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self { conn })
    }

    pub async fn set_value<T>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(value)?;
        self.conn.set::<&str, String, ()>(key, serialized).await?;
        Ok(())
    }

    pub async fn get_value<T>(&mut self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
    {
        let result: Option<String> = self.conn.get(key).await?;
        match result {
            Some(serialized) => {
                let value: T = serde_json::from_str(&serialized)?;
                Ok(value)
            }
            None => Err("serilization".into()),
        }
    }

    pub async fn get_values<T>(&mut self, prefix: &str) -> Result<Vec<T>, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        let keys: Vec<String> = self.conn.keys(format!("{prefix}*")).await?;
        let response: Vec<Option<String>> = self.conn.mget(keys).await?;
        let values: Vec<String> = response.into_iter().flatten().collect();

        let mut results: Vec<T> = Vec::new();
        for result in values {
            let value: T = serde_json::from_str(&result)?;
            results.push(value);
        }

        Ok(results)
    }

    pub async fn get_keys(&mut self, prefix: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let keys: Vec<String> = self.conn.keys(format!("{prefix}*")).await?;
        Ok(keys)
    }
}
