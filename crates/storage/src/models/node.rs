use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Node {
    pub id: i32,
    pub chain: String,
    pub url: String,
    pub status: String,
    pub priority: i32,
}
