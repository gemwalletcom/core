use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chart {
    pub coin_id: String,
    pub price: f64,
    pub ts: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_hourly_avg)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyChart {
    pub coin_id: String,
    pub price: f64,
    pub ts: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_daily_avg)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DailyChart {
    pub coin_id: String,
    pub price: f64,
    pub ts: NaiveDateTime,
}

impl From<HourlyChart> for Chart {
    fn from(hourly_chart: HourlyChart) -> Self {
        Chart {
            coin_id: hourly_chart.coin_id,
            price: hourly_chart.price,
            ts: hourly_chart.ts,
        }
    }
}

impl From<DailyChart> for Chart {
    fn from(daily_chart: DailyChart) -> Self {
        Chart {
            coin_id: daily_chart.coin_id,
            price: daily_chart.price,
            ts: daily_chart.ts,
        }
    }
}
