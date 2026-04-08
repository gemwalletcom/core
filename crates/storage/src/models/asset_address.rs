use diesel::prelude::*;
use primitives::{AssetAddress, AssetId as PrimitiveAssetId, Chain};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

pub trait AssetAddressRowsExt {
    fn asset_ids(self) -> Vec<PrimitiveAssetId>;
}

impl AssetAddressRowsExt for Vec<AssetAddressRow> {
    fn asset_ids(self) -> Vec<PrimitiveAssetId> {
        let mut seen = HashSet::new();

        self.into_iter().filter_map(|row| seen.insert(row.asset_id.0.clone()).then_some(row.asset_id.0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Asset;

    #[test]
    fn test_asset_ids() {
        let rows = vec![
            AssetAddressRow::new(Chain::Ethereum, Asset::mock_eth().id, "0xwallet".to_string(), Some("100".to_string())),
            AssetAddressRow::new(Chain::Ethereum, Asset::mock_ethereum_usdc().id.clone(), "0xwallet".to_string(), Some("1".to_string())),
            AssetAddressRow::new(Chain::Ethereum, Asset::mock_ethereum_usdc().id.clone(), "0xother".to_string(), Some("2".to_string())),
        ];

        assert_eq!(rows.asset_ids(), vec![Asset::mock_eth().id, Asset::mock_ethereum_usdc().id]);
    }
}
