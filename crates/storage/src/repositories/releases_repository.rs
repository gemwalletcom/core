use crate::DatabaseError;

use crate::database::releases::ReleasesStore;
use crate::models::Release;
use crate::DatabaseClient;

pub trait ReleasesRepository {
    fn get_releases(&mut self) -> Result<Vec<Release>, DatabaseError>;
    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, DatabaseError>;
    fn update_release(&mut self, release: Release) -> Result<usize, DatabaseError>;
}

impl ReleasesRepository for DatabaseClient {
    fn get_releases(&mut self) -> Result<Vec<Release>, DatabaseError> {
        Ok(ReleasesStore::get_releases(self)?)
    }

    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, DatabaseError> {
        Ok(ReleasesStore::add_releases(self, values)?)
    }

    fn update_release(&mut self, release: Release) -> Result<usize, DatabaseError> {
        Ok(ReleasesStore::update_release(self, release)?)
    }
}
