use std::str::FromStr;

use diesel::prelude::*;
use primitives::PlatformStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::releases)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Release {
    pub platform_store: String,
    pub version: String,
    pub upgrade_required: bool,
}

impl Release {
    pub fn as_pritmitive(&self) -> primitives::Release {
        primitives::Release {
            store: PlatformStore::from_str(&self.platform_store).unwrap(),
            version: self.version.clone(),
            upgrade_required: self.upgrade_required,
        }
    }

    pub fn from_primitive(release: primitives::Release) -> Self {
        Self {
            platform_store: release.store.as_ref().to_string(),
            version: release.version,
            upgrade_required: release.upgrade_required,
        }
    }
}
