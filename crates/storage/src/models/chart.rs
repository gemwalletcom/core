use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chart {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_hourly)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyChart {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_daily)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DailyChart {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

impl From<HourlyChart> for Chart {
    fn from(chart: HourlyChart) -> Self {
        Chart {
            coin_id: chart.coin_id,
            price: chart.price,
            created_at: chart.created_at,
        }
    }
}

impl From<DailyChart> for Chart {
    fn from(chart: DailyChart) -> Self {
        Chart {
            coin_id: chart.coin_id,
            price: chart.price,
            created_at: chart.created_at,
        }
    }
}
