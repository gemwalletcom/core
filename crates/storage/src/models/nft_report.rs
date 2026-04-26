use diesel::prelude::*;

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::nft_reports)]
pub struct NewNftReportRow {
    pub device_id: i32,
    pub collection_id: i32,
    pub asset_id: Option<i32>,
    pub reason: Option<String>,
}
