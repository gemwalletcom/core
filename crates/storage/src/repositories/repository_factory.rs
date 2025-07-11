use crate::{AssetsAddressesRepository, AssetsLinksRepository, AssetsRepository, AssetsTypesRepository, ChartsRepository, DatabaseClient, DevicesRepository, FiatRepository, LinkTypesRepository, NftRepository, NodesRepository, ParserStateRepository, SubscriptionsRepository};

pub struct RepositoryFactory<'a> {
    database: &'a mut DatabaseClient,
}

impl<'a> RepositoryFactory<'a> {
    pub fn new(database: &'a mut DatabaseClient) -> Self {
        Self { database }
    }

    pub fn subscriptions(&mut self) -> &mut dyn SubscriptionsRepository {
        self.database
    }

    pub fn assets(&mut self) -> &mut dyn AssetsRepository {
        self.database
    }

    pub fn assets_addresses(&mut self) -> &mut dyn AssetsAddressesRepository {
        self.database
    }

    pub fn assets_links(&mut self) -> &mut dyn AssetsLinksRepository {
        self.database
    }

    pub fn assets_types(&mut self) -> &mut dyn AssetsTypesRepository {
        self.database
    }

    pub fn charts(&mut self) -> &mut dyn ChartsRepository {
        self.database
    }

    pub fn devices(&mut self) -> &mut dyn DevicesRepository {
        self.database
    }

    pub fn fiat(&mut self) -> &mut dyn FiatRepository {
        self.database
    }

    pub fn link_types(&mut self) -> &mut dyn LinkTypesRepository {
        self.database
    }

    pub fn nft(&mut self) -> &mut dyn NftRepository {
        self.database
    }

    pub fn nodes(&mut self) -> &mut dyn NodesRepository {
        self.database
    }

    pub fn parser_state(&mut self) -> &mut dyn ParserStateRepository {
        self.database
    }
}

// Extension trait for DatabaseClient
pub trait DatabaseClientExt {
    fn repositories(&mut self) -> RepositoryFactory;
}

impl DatabaseClientExt for DatabaseClient {
    fn repositories(&mut self) -> RepositoryFactory {
        RepositoryFactory::new(self)
    }
}