use diesel::prelude::*;
use primitives::{AssetAddress, AssetId as PrimitiveAssetId, Chain};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetId, ChainRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetAddressRow {
    pub chain: ChainRow,
    pub asset_id: AssetId,
    pub address: String,
    pub value: Option<String>,
}

impl AssetAddressRow {
    pub fn new(chain: Chain, asset_id: PrimitiveAssetId, address: String, value: Option<String>) -> Self {
        Self {
            chain: ChainRow::from(chain),
            asset_id: asset_id.into(),
            address,
            value,
        }
    }

    pub fn from_primitive(asset_address: AssetAddress) -> Self {
        Self {
            chain: ChainRow::from(asset_address.asset_id.chain),
            asset_id: asset_address.asset_id.into(),
            address: asset_address.address.clone(),
            value: asset_address.value.clone(),
        }
    }

    pub fn as_primitive(&self) -> AssetAddress {
        AssetAddress {
            asset_id: self.asset_id.0.clone(),
            address: self.address.clone(),
            value: self.value.clone(),
        }
    }
}
