use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::migrations::MigrationsStore;

pub trait MigrationsRepository {
    fn run_migrations(&mut self) -> Result<(), DatabaseError>;
}

impl MigrationsRepository for DatabaseClient {
    fn run_migrations(&mut self) -> Result<(), DatabaseError> {
        MigrationsStore::run_migrations(self);
        Ok(())
    }
}
