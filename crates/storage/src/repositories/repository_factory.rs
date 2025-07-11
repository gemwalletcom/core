use crate::{AssetsAddressesRepository, DatabaseClient, SubscriptionsRepository};

pub struct RepositoryFactory<'a> {
    database: &'a mut DatabaseClient,
}

impl<'a> RepositoryFactory<'a> {
    pub fn new(database: &'a mut DatabaseClient) -> Self {
        Self { database }
    }

    pub fn subscriptions(&mut self) -> SubscriptionsRepository {
        SubscriptionsRepository::new(self.database)
    }

    pub fn assets_addresses(&mut self) -> &mut dyn AssetsAddressesRepository {
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