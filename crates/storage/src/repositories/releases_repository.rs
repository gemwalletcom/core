use std::error::Error;

use crate::DatabaseClient;
use crate::database::releases::ReleasesStore;
use crate::models::Release;

pub trait ReleasesRepository {
    fn get_releases(&mut self) -> Result<Vec<Release>, Box<dyn Error + Send + Sync>>;
    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_release(&mut self, release: Release) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl ReleasesRepository for DatabaseClient {
    fn get_releases(&mut self) -> Result<Vec<Release>, Box<dyn Error + Send + Sync>> {
        Ok(ReleasesStore::get_releases(self)?)
    }

    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ReleasesStore::add_releases(self, values)?)
    }

    fn update_release(&mut self, release: Release) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ReleasesStore::update_release(self, release)?)
    }
}