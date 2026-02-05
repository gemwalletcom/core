use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::assets_usage_ranks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetUsageRankRow {
    pub asset_id: String,
    pub usage_rank: i32,
}
