use diesel::prelude::*;
use primitives::{AssetAddress, AssetId, Chain};
use serde::{Deserialize, Serialize};

use crate::sql_types::ChainRow;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetAddressRow {
    pub chain: ChainRow,
    pub asset_id: String,
    pub address: String,
    pub value: Option<String>,
}

impl AssetAddressRow {
    pub fn new(chain: Chain, asset_id: String, address: String, value: Option<String>) -> Self {
        Self {
            chain: ChainRow::from(chain),
            asset_id,
            address,
            value,
        }
    }

    pub fn from_primitive(asset_address: AssetAddress) -> Self {
        Self {
            chain: ChainRow::from(asset_address.asset_id.chain),
            asset_id: asset_address.asset_id.to_string(),
            address: asset_address.address.clone(),
            value: asset_address.value.clone(),
        }
    }

    pub fn as_primitive(&self) -> AssetAddress {
        AssetAddress {
            asset_id: AssetId::new(&self.asset_id).unwrap(),
            address: self.address.clone(),
            value: self.value.clone(),
        }
    }
}
