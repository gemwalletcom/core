use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::operator_appstore_positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppStorePosition {
    pub store: String,
    pub app: String,
    pub keyword: String,
    pub country: String,
    pub position: i32,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::operator_appstore_information)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppStoreInformation {
    pub store: String,
    pub app: String,
    pub country: String,
    pub title: String,
    pub version: String,
    pub ratings: Option<f64>,
    pub average_rating: Option<f64>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::operator_appstore_reviews)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppStoreReview {
    pub review_id: String,
    pub store: String,
    pub app: String,
    pub country: String,
    pub title: String,
    pub content: String,
    pub version: String,
    pub author: String,
    pub rating: i32,
}
