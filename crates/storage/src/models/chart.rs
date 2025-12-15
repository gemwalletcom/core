use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::ChartData;
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
    pub fn from_price(price: PriceRow) -> Self {
        ChartRow {
            coin_id: price.id,
            price: price.price,
            created_at: price.last_updated_at,
        }
    }

    pub fn as_chart_data(&self) -> ChartData {
        ChartData {
            coin_id: self.coin_id.clone(),
            price: self.price,
            created_at: self.created_at.and_utc(),
        }
    }

    pub fn from_chart_data(data: ChartData) -> Self {
        ChartRow {
            coin_id: data.coin_id,
            price: data.price,
            created_at: data.created_at.naive_utc(),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::charts_hourly)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyChartRow {
    pub coin_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable)]
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
