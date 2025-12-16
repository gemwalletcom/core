use std::str::FromStr;

use diesel::prelude::*;
use primitives::{PlatformStore, Release};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::releases)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReleaseRow {
    pub platform_store: String,
    pub version: String,
    pub upgrade_required: bool,
}

impl ReleaseRow {
    pub fn as_pritmitive(&self) -> Release {
        Release {
            store: PlatformStore::from_str(&self.platform_store).unwrap(),
            version: self.version.clone(),
            upgrade_required: self.upgrade_required,
        }
    }

    pub fn from_primitive(release: Release) -> Self {
        Self {
            platform_store: release.store.as_ref().to_string(),
            version: release.version,
            upgrade_required: release.upgrade_required,
        }
    }
}
