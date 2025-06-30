use crate::models::{Chart, DailyChart, HourlyChart};
use crate::schema::charts::dsl::{charts, coin_id, ts};
use crate::schema::charts_daily_avg::dsl::{charts_daily_avg, coin_id as daily_coin_id, ts as daily_ts};
use crate::schema::charts_hourly_avg::dsl::{charts_hourly_avg, coin_id as hourly_coin_id, ts as hourly_ts};
use crate::DatabaseClient;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;

pub enum ChartGranularity {
    Minute,
    Hourly,
    Daily,
}

impl DatabaseClient {
    pub async fn add_charts(&mut self, values: Vec<Chart>) -> Result<usize, Error> {
        diesel::insert_into(charts).values(values).execute(&mut self.connection)
    }

    pub async fn get_charts(
        &mut self,
        target_coin_id: String,
        interval: ChartGranularity,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<Chart>, Error> {
        match interval {
            ChartGranularity::Minute => charts
                .filter(coin_id.eq(target_coin_id.clone()))
                .filter(ts.between(from, to))
                .order(ts.asc())
                .load::<Chart>(&mut self.connection)
                .map(|x| x.into_iter().collect()),
            ChartGranularity::Hourly => charts_hourly_avg
                .filter(hourly_coin_id.eq(target_coin_id.clone()))
                .filter(hourly_ts.between(from, to))
                .order(hourly_ts.asc())
                .load::<HourlyChart>(&mut self.connection)
                .map(|x| x.into_iter().map(Chart::from).collect()),
            ChartGranularity::Daily => charts_daily_avg
                .filter(daily_coin_id.eq(target_coin_id.clone()))
                .filter(daily_ts.between(from, to))
                .order(daily_ts.asc())
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

    pub async fn aggregate_charts(&mut self) -> Result<usize, diesel::result::Error> {
        diesel::sql_query("SELECT aggregate_charts();").execute(&mut self.connection)
    }

    pub async fn cleanup_old_charts_data(&mut self) -> Result<usize, diesel::result::Error> {
        diesel::sql_query("SELECT cleanup_old_charts_data();").execute(&mut self.connection)
    }
}
