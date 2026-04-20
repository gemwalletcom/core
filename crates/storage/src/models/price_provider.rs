use diesel::prelude::*;
use primitives::PriceProvider;
use serde::{Deserialize, Serialize};

use crate::sql_types::PriceProviderRow;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceProviderConfigRow {
    pub id: PriceProviderRow,
    pub enabled: bool,
    pub priority: i32,
}

impl PriceProviderConfigRow {
    pub fn new(provider: PriceProvider, enabled: bool) -> Self {
        Self {
            id: provider.into(),
            enabled,
            priority: provider.priority(),
        }
    }
}
