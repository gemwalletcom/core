use diesel::prelude::*;
use primitives::{AssetAddress, AssetId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetAddressRow {
    pub chain: String,
    pub asset_id: String,
    pub address: String,
    pub value: Option<String>,
}

impl AssetAddressRow {
    pub fn new(chain: String, asset_id: String, address: String, value: Option<String>) -> Self {
        Self { chain, asset_id, address, value }
    }

    pub fn from_primitive(asset_address: AssetAddress) -> Self {
        Self {
            chain: asset_address.asset_id.chain.as_ref().to_string(),
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
