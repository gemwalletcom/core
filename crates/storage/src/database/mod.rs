pub mod assets;
pub mod assets_addresses;
pub mod assets_links;
pub mod assets_types;

pub mod charts;
pub mod devices;
pub mod fiat;
pub mod link_types;
pub mod migrations;
pub mod nft;
pub mod nodes;
pub mod parser_state;
pub mod perpetuals;
pub mod price_alerts;
pub mod prices;
pub mod prices_dex;
pub mod releases;
pub mod scan_addresses;
pub mod subscriptions;
pub mod support;
pub mod tag;
pub mod transactions;

use diesel::Connection;
use diesel::pg::PgConnection;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");

use crate::{
    AssetsAddressesRepository, AssetsLinksRepository, AssetsRepository, AssetsTypesRepository, ChartsRepository, DevicesRepository, FiatRepository,
    LinkTypesRepository, MigrationsRepository, NftRepository, NodesRepository, ParserStateRepository, PerpetualsRepository, PriceAlertsRepository,
    PricesDexRepository, PricesRepository, ReleasesRepository, ScanAddressesRepository, SubscriptionsRepository, SupportRepository, TagRepository,
    TransactionsRepository,
};

pub struct DatabaseClient {
    connection: PgConnection,
}

impl DatabaseClient {
    pub fn new(database_url: &str) -> Self {
        let connection = PgConnection::establish(database_url).unwrap_or_else(|_| panic!("Error connecting to {database_url}"));

        Self { connection }
    }

    // Direct repository access methods
    pub fn assets(&mut self) -> &mut dyn AssetsRepository {
        self
    }

    pub fn assets_addresses(&mut self) -> &mut dyn AssetsAddressesRepository {
        self
    }

    pub fn assets_links(&mut self) -> &mut dyn AssetsLinksRepository {
        self
    }

    pub fn assets_types(&mut self) -> &mut dyn AssetsTypesRepository {
        self
    }

    pub fn charts(&mut self) -> &mut dyn ChartsRepository {
        self
    }

    pub fn devices(&mut self) -> &mut dyn DevicesRepository {
        self
    }

    pub fn fiat(&mut self) -> &mut dyn FiatRepository {
        self
    }

    pub fn link_types(&mut self) -> &mut dyn LinkTypesRepository {
        self
    }

    pub fn migrations(&mut self) -> &mut dyn MigrationsRepository {
        self
    }

    pub fn perpetuals(&mut self) -> &mut dyn PerpetualsRepository {
        self
    }

    pub fn nft(&mut self) -> &mut dyn NftRepository {
        self
    }

    pub fn nodes(&mut self) -> &mut dyn NodesRepository {
        self
    }

    pub fn parser_state(&mut self) -> &mut dyn ParserStateRepository {
        self
    }

    pub fn price_alerts(&mut self) -> &mut dyn PriceAlertsRepository {
        self
    }

    pub fn prices(&mut self) -> &mut dyn PricesRepository {
        self
    }

    pub fn prices_dex(&mut self) -> &mut dyn PricesDexRepository {
        self
    }

    pub fn releases(&mut self) -> &mut dyn ReleasesRepository {
        self
    }

    pub fn scan_addresses(&mut self) -> &mut dyn ScanAddressesRepository {
        self
    }

    pub fn subscriptions(&mut self) -> &mut dyn SubscriptionsRepository {
        self
    }

    pub fn support(&mut self) -> &mut dyn SupportRepository {
        self
    }

    pub fn tag(&mut self) -> &mut dyn TagRepository {
        self
    }

    pub fn transactions(&mut self) -> &mut dyn TransactionsRepository {
        self
    }
}
