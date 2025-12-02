use diesel::prelude::*;

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::nft_reports)]
pub struct NewNftReport {
    pub device_id: i32,
    pub collection_id: String,
    pub asset_id: Option<String>,
    pub reason: Option<String>,
}
