pub mod database;
pub mod error;
pub mod redis_client;
pub mod models;
pub mod repositories;
pub mod schema;

pub use self::database::{
    DatabaseClient,
    assets::{AssetFilter, AssetUpdate},
};
pub use self::error::DatabaseError;
pub use self::redis_client::RedisClient;
pub use self::models::ScanAddressType;
pub use self::repositories::{
    assets_addresses_repository::AssetsAddressesRepository, assets_links_repository::AssetsLinksRepository, assets_repository::AssetsRepository,
    assets_types_repository::AssetsTypesRepository, charts_repository::ChartsRepository, devices_repository::DevicesRepository,
    fiat_repository::FiatRepository, link_types_repository::LinkTypesRepository, migrations_repository::MigrationsRepository, nft_repository::NftRepository,
    nodes_repository::NodesRepository, parser_state_repository::ParserStateRepository, perpetuals_repository::PerpetualsRepository,
    price_alerts_repository::PriceAlertsRepository, prices_dex_repository::PricesDexRepository, prices_repository::PricesRepository,
    releases_repository::ReleasesRepository, scan_addresses_repository::ScanAddressesRepository, subscriptions_repository::SubscriptionsRepository,
    support_repository::SupportRepository, tag_repository::TagRepository, transactions_repository::TransactionsRepository,
};

#[derive(Clone)]
pub struct Database(database::PgPool);

impl Database {
    pub fn new(database_url: &str, pool_size: u32) -> Self {
        Self(database::create_pool(database_url, pool_size))
    }

    pub fn client(&self) -> Result<DatabaseClient, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DatabaseClient::from_pool(&self.0)?)
    }
}
