use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::fiat_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatAsset {
    pub id: i32,
    pub asset: String,
    pub provider: String,
    pub symbol: String,
    pub network: Option<String>,
}