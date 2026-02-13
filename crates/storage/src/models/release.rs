use crate::sql_types::PlatformStore;
use diesel::prelude::*;
use primitives::Release;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::releases)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReleaseRow {
    pub platform_store: PlatformStore,
    pub version: String,
    pub upgrade_required: bool,
    pub update_enabled: bool,
}

impl ReleaseRow {
    pub fn as_primitive(&self) -> Release {
        Release {
            store: self.platform_store.0,
            version: self.version.clone(),
            upgrade_required: self.upgrade_required,
        }
    }

    pub fn from_primitive(release: Release) -> Self {
        Self {
            platform_store: release.store.into(),
            version: release.version,
            upgrade_required: release.upgrade_required,
            update_enabled: true,
        }
    }
}
