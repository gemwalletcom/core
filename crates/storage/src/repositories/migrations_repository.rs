use crate::database::migrations::MigrationsStore;
use crate::DatabaseClient;
use std::error::Error;

pub trait MigrationsRepository {
    fn run_migrations(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl MigrationsRepository for DatabaseClient {
    fn run_migrations(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        MigrationsStore::run_migrations(self);
        Ok(())
    }
}