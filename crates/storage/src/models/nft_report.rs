use diesel::prelude::*;

use crate::sql_types::AssetId;

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::nft_reports)]
pub struct NewNftReportRow {
    pub device_id: i32,
    pub collection_id: String,
    pub asset_id: Option<AssetId>,
    pub reason: Option<String>,
}
