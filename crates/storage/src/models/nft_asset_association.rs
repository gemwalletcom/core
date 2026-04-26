use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::nft_assets_associations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NftAssetAssociationRow {
    pub id: i32,
    pub address_id: i32,
    pub asset_id: i32,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::nft_assets_associations)]
pub struct NewNftAssetAssociationRow {
    pub address_id: i32,
    pub asset_id: i32,
}
