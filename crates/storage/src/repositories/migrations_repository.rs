use crate::DatabaseClient;
use crate::database::migrations::MigrationsStore;

pub trait MigrationsRepository {
    fn migrations(&mut self);
}

impl MigrationsRepository for DatabaseClient {
    fn migrations(&mut self) {
        MigrationsStore::migrations(self);
    }
}