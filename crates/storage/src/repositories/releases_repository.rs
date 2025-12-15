use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::releases::ReleasesStore;
use crate::models::ReleaseRow;

pub trait ReleasesRepository {
    fn get_releases(&mut self) -> Result<Vec<ReleaseRow>, DatabaseError>;
    fn add_releases(&mut self, values: Vec<ReleaseRow>) -> Result<usize, DatabaseError>;
    fn update_release(&mut self, release: ReleaseRow) -> Result<usize, DatabaseError>;
}

impl ReleasesRepository for DatabaseClient {
    fn get_releases(&mut self) -> Result<Vec<ReleaseRow>, DatabaseError> {
        Ok(ReleasesStore::get_releases(self)?)
    }

    fn add_releases(&mut self, values: Vec<ReleaseRow>) -> Result<usize, DatabaseError> {
        Ok(ReleasesStore::add_releases(self, values)?)
    }

    fn update_release(&mut self, release: ReleaseRow) -> Result<usize, DatabaseError> {
        Ok(ReleasesStore::update_release(self, release)?)
    }
}
