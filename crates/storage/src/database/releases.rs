use crate::{models::*, DatabaseClient};

use diesel::{prelude::*, upsert::excluded};

pub(crate) trait ReleasesStore {
    fn get_releases(&mut self) -> Result<Vec<Release>, diesel::result::Error>;
    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, diesel::result::Error>;
    fn update_release(&mut self, release: Release) -> Result<usize, diesel::result::Error>;
}

impl ReleasesStore for DatabaseClient {
    fn get_releases(&mut self) -> Result<Vec<Release>, diesel::result::Error> {
        use crate::schema::releases::dsl::*;
        releases.order(updated_at.desc()).select(Release::as_select()).load(&mut self.connection)
    }

    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, diesel::result::Error> {
        use crate::schema::releases::dsl::*;
        diesel::insert_into(releases)
            .values(&values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn update_release(&mut self, release: Release) -> Result<usize, diesel::result::Error> {
        use crate::schema::releases::dsl::*;
        diesel::insert_into(releases)
            .values(&release)
            .on_conflict(platform_store)
            .do_update()
            .set(version.eq(excluded(version)))
            .execute(&mut self.connection)
    }
}
