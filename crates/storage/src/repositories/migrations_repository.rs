use crate::database::migrations::MigrationsStore;
use crate::DatabaseClient;
use crate::DatabaseError;

pub trait MigrationsRepository {
    fn run_migrations(&mut self) -> Result<(), DatabaseError>;
}

impl MigrationsRepository for DatabaseClient {
    fn run_migrations(&mut self) -> Result<(), DatabaseError> {
        MigrationsStore::run_migrations(self);
        Ok(())
    }
}
