use crate::DatabaseClient;
use diesel_migrations::{MigrationHarness};
use super::{MIGRATIONS};

pub(crate) trait MigrationsStore {
    fn migrations(&mut self);
}

impl MigrationsStore for DatabaseClient {
    fn migrations(&mut self) {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
}