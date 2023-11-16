use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::swap_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SwapAsset {
    pub id: i32,
    pub asset_id: String,
}