use diesel::prelude::*;
use diesel::result::Error;
use diesel::pg::PgConnection;
use chrono::NaiveDateTime;
use crate::models::{Chart, HourlyChart, DailyChart};
use crate::schema::charts::dsl::{charts, coin_id, ts};
use crate::schema::charts_hourly_avg::dsl::{charts_hourly_avg, coin_id as hourly_coin_id, ts as hourly_ts};
use crate::schema::charts_daily_avg::dsl::{charts_daily_avg, coin_id as daily_coin_id, ts as daily_ts};

pub enum ChartGranularity {
    Minute,
    Hourly,
    Daily,
}

pub async fn get_charts(
    conn: &mut PgConnection,
    target_coin_id: String,
    interval: ChartGranularity,
    from: NaiveDateTime,
    to: NaiveDateTime,
) -> Result<Vec<Chart>, Error> {
    match interval {
        ChartGranularity::Minute => {
            charts
                .filter(coin_id.eq(target_coin_id.clone()))
                .filter(ts.between(from, to))
                .order(ts.asc())
                .load::<Chart>(conn)
        }
        ChartGranularity::Hourly => {
            charts_hourly_avg
                .filter(hourly_coin_id.eq(target_coin_id.clone()))
                .filter(hourly_ts.between(from, to))
                .order(hourly_ts.asc())
                .load::<HourlyChart>(conn)
                .map(|loaded_charts| loaded_charts.into_iter().map(Chart::from).collect())
        }
        ChartGranularity::Daily => {
            charts_daily_avg
                .filter(daily_coin_id.eq(target_coin_id.clone()))
                .filter(daily_ts.between(from.date(), to.date()))
                .order(daily_ts.asc())
                .load::<DailyChart>(conn)
                .map(|loaded_charts| loaded_charts.into_iter().map(Chart::from).collect())
        }
    }
}