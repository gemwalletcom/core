use crate::models::{Chart, DailyChart, HourlyChart};
use crate::schema::charts::dsl::{charts, coin_id, created_at};
use crate::schema::charts_daily::dsl::{charts_daily, coin_id as daily_coin_id, created_at as daily_created_at};
use crate::schema::charts_hourly::dsl::{charts_hourly, coin_id as hourly_coin_id, created_at as hourly_created_at};
use crate::DatabaseClient;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;

pub enum ChartGranularity {
    Minute,
    Minute15,
    Hourly,
    Hour6,
    Daily,
}

impl ChartGranularity {
    pub fn as_interval_string(&self) -> &str {
        match self {
            ChartGranularity::Minute => "1 minute",
            ChartGranularity::Minute15 => "15 minutes",
            ChartGranularity::Hourly => "1 hour",
            ChartGranularity::Hour6 => "6 hours",
            ChartGranularity::Daily => "1 day",
        }
    }
}

impl DatabaseClient {
    pub async fn add_charts(&mut self, values: Vec<Chart>) -> Result<usize, Error> {
        diesel::insert_into(charts)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub async fn get_charts(
        &mut self,
        target_coin_id: String,
        interval: ChartGranularity,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<Chart>, Error> {
        match interval {
            ChartGranularity::Minute | ChartGranularity::Minute15 => charts
                .filter(coin_id.eq(target_coin_id.clone()))
                .filter(created_at.between(from, to))
                .order(created_at.asc())
                .load::<Chart>(&mut self.connection)
                .map(|x| x.into_iter().collect()),
            ChartGranularity::Hourly | ChartGranularity::Hour6 => charts_hourly
                .filter(hourly_coin_id.eq(target_coin_id.clone()))
                .filter(hourly_created_at.between(from, to))
                .order(hourly_created_at.asc())
                .load::<HourlyChart>(&mut self.connection)
                .map(|x| x.into_iter().map(Chart::from).collect()),
            ChartGranularity::Daily => charts_daily
                .filter(daily_coin_id.eq(target_coin_id.clone()))
                .filter(daily_created_at.between(from, to))
                .order(daily_created_at.asc())
                .load::<DailyChart>(&mut self.connection)
                .map(|x| x.into_iter().map(Chart::from).collect()),
        }
    }

    pub async fn aggregate_hourly_charts(&mut self) -> Result<usize, diesel::result::Error> {
        diesel::sql_query("SELECT aggregate_hourly_charts();").execute(&mut self.connection)
    }

    pub async fn aggregate_daily_charts(&mut self) -> Result<usize, diesel::result::Error> {
        diesel::sql_query("SELECT aggregate_daily_charts();").execute(&mut self.connection)
    }

    pub async fn cleanup_charts_data(&mut self) -> Result<usize, diesel::result::Error> {
        diesel::sql_query("SELECT cleanup_all_charts_data();").execute(&mut self.connection)
    }
}
