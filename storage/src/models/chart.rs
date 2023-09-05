use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chart {
    pub coin_id: String,
    pub price: f64,
    pub date: NaiveDateTime,
    pub market_cap: f64,
    pub volume: f64,
}

impl PartialEq for Chart {
    fn eq(&self, other: &Self) -> bool {
        self.coin_id == other.coin_id && self.date == other.date
    }
}
impl Eq for Chart {}

impl Hash for Chart {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.coin_id.hash(state);
        self.date.hash(state);
    }
}

pub type ChartResult = (chrono::NaiveDateTime, f64);