use diesel::prelude::*;
use chrono::NaiveDate;
use chrono::NaiveDateTime;

// Schema for the raw charts table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chart {
    pub coin_id: String,
    pub price: f32,
    pub ts: NaiveDateTime,
}

// For inserting new raw chart data
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChart {
    pub coin_id: String,
    pub price: f32,
    pub ts: NaiveDateTime,
}

// Schema for hourly aggregated charts
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_hourly_avg)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyChart {
    pub coin_id: String,
    pub price: f32,
    pub ts: NaiveDateTime,
}

// Schema for daily aggregated charts
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_daily_avg)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DailyChart {
    pub coin_id: String,
    pub price: f32,
    pub ts: NaiveDate,
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
            ts: daily_chart.ts.and_hms_opt(0, 0, 0).unwrap(),
        }
    }
}
