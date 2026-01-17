use std::error::Error;

mod config_cacher;
pub mod database;
pub mod error;
pub mod models;
pub mod repositories;
pub mod schema;
pub mod sql_types;

pub use config_cacher::ConfigCacher;

diesel::allow_columns_to_appear_in_same_group_by_clause!(schema::transactions_addresses::address, schema::transactions::chain,);

pub use self::database::{
    DatabaseClient,
    assets::{AssetFilter, AssetUpdate},
    rewards::{AbusePatterns, ReferralUpdate},
    rewards_redemptions::RedemptionUpdate,
};
pub use self::error::{DatabaseError, ReferralValidationError};
pub use self::models::{NewNotificationRow, NewWalletRow, RewardRedemptionOptionRow};
pub use self::repositories::{
    assets_addresses_repository::AssetsAddressesRepository, assets_links_repository::AssetsLinksRepository, assets_repository::AssetsRepository,
    chains_repository::ChainsRepository, charts_repository::ChartsRepository, config_repository::ConfigRepository, devices_repository::DevicesRepository,
    fiat_repository::FiatRepository, migrations_repository::MigrationsRepository, nft_repository::NftRepository,
    notifications_repository::NotificationsRepository, parser_state_repository::ParserStateRepository, perpetuals_repository::PerpetualsRepository,
    price_alerts_repository::PriceAlertsRepository, prices_dex_repository::PricesDexRepository, prices_repository::PricesRepository,
    releases_repository::ReleasesRepository, rewards_redemptions_repository::RewardsRedemptionsRepository, rewards_repository::RewardsRepository,
    risk_signals_repository::RiskSignalsRepository, scan_addresses_repository::ScanAddressesRepository, subscriptions_repository::SubscriptionsRepository,
    support_repository::SupportRepository, tag_repository::TagRepository, transactions_repository::TransactionsRepository,
    wallets_repository::WalletsRepository,
};
pub use self::sql_types::{NotificationType, WalletSource, WalletType};
pub use diesel::OptionalExtension;

#[derive(Clone)]
pub struct Database(database::PgPool);

impl Database {
    pub fn new(database_url: &str, pool_size: u32) -> Self {
        Self(database::create_pool(database_url, pool_size))
    }

    pub fn client(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        Ok(DatabaseClient::from_pool(&self.0)?)
    }
}

impl Database {
    pub fn assets(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn assets_addresses(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn assets_links(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn chains(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn charts(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn devices(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn fiat(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn migrations(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn perpetuals(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn nft(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn notifications(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn parser_state(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn price_alerts(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn prices(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn prices_dex(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn rewards(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn rewards_redemptions(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn releases(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn scan_addresses(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn subscriptions(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn support(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn tag(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn transactions(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }

    pub fn wallets(&self) -> Result<DatabaseClient, Box<dyn Error + Send + Sync>> {
        self.client()
    }
}
