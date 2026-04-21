use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::PriceRow;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChartRow {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

impl ChartRow {
    pub fn new(coin_id: String, price: f64, created_at: NaiveDateTime) -> Self {
        ChartRow { coin_id, price, created_at }
    }

    pub fn from_price(price: PriceRow) -> Self {
        ChartRow {
            coin_id: price.id,
            price: price.price,
            created_at: price.last_updated_at,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::charts_hourly)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyChartRow {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::charts_daily)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DailyChartRow {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

impl From<HourlyChartRow> for ChartRow {
    fn from(chart: HourlyChartRow) -> Self {
        ChartRow {
            coin_id: chart.coin_id,
            price: chart.price,
            created_at: chart.created_at,
        }
    }
}

impl From<DailyChartRow> for ChartRow {
    fn from(chart: DailyChartRow) -> Self {
        ChartRow {
            coin_id: chart.coin_id,
            price: chart.price,
            created_at: chart.created_at,
        }
    }
}

impl From<ChartRow> for HourlyChartRow {
    fn from(chart: ChartRow) -> Self {
        HourlyChartRow {
            coin_id: chart.coin_id,
            price: chart.price,
            created_at: chart.created_at,
        }
    }
}

impl From<ChartRow> for DailyChartRow {
    fn from(chart: ChartRow) -> Self {
        DailyChartRow {
            coin_id: chart.coin_id,
            price: chart.price,
            created_at: chart.created_at,
        }
    }
}
