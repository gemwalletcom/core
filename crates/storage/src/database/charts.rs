use crate::models::Chart;
use crate::schema::charts::dsl::{charts, coin_id};
use crate::schema::charts_daily::dsl::{charts_daily, coin_id as daily_coin_id};
use crate::schema::charts_hourly::dsl::{charts_hourly, coin_id as hourly_coin_id};
use crate::DatabaseClient;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::result::Error;
use primitives::ChartPeriod;

pub enum ChartGranularity {
    Minute,
    Minute15,
    Hourly,
    Hour6,
    Daily,
}

pub type ChartResult = (chrono::NaiveDateTime, f64);

impl DatabaseClient {
    pub async fn add_charts(&mut self, values: Vec<Chart>) -> Result<usize, Error> {
        diesel::insert_into(charts)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub async fn get_charts(&mut self, target_coin_id: String, period: &ChartPeriod) -> Result<Vec<ChartResult>, Error> {
        let date_selection = format!("date_bin('{}', created_at, timestamp '2000-01-01')", self.period_sql(period.clone()));
        let granularity = Self::get_chart_granularity_for_period(period);
        let created_at_filter = format!("created_at >= now() - INTERVAL '{} minutes'", self.period_minutes(period.clone()));
        match granularity {
            ChartGranularity::Minute | ChartGranularity::Minute15 => charts
                .select((
                    sql::<diesel::sql_types::Timestamp>(date_selection.as_str()),
                    sql::<diesel::sql_types::Double>("AVG(price)"),
                ))
                .filter(coin_id.eq(target_coin_id))
                .filter(sql::<diesel::sql_types::Bool>(&created_at_filter))
                .group_by(sql::<diesel::sql_types::Numeric>("1"))
                .order(sql::<diesel::sql_types::Numeric>("1").desc())
                .load(&mut self.connection),
            ChartGranularity::Hourly | ChartGranularity::Hour6 => charts_hourly
                .select((
                    sql::<diesel::sql_types::Timestamp>(date_selection.as_str()),
                    sql::<diesel::sql_types::Double>("AVG(price)"),
                ))
                .filter(hourly_coin_id.eq(target_coin_id))
                .filter(sql::<diesel::sql_types::Bool>(&created_at_filter))
                .group_by(sql::<diesel::sql_types::Numeric>("1"))
                .order(sql::<diesel::sql_types::Numeric>("1").desc())
                .load(&mut self.connection),
            ChartGranularity::Daily => charts_daily
                .select((
                    sql::<diesel::sql_types::Timestamp>(date_selection.as_str()),
                    sql::<diesel::sql_types::Double>("AVG(price)"),
                ))
                .filter(daily_coin_id.eq(target_coin_id))
                .filter(sql::<diesel::sql_types::Bool>(&created_at_filter))
                .group_by(sql::<diesel::sql_types::Numeric>("1"))
                .order(sql::<diesel::sql_types::Numeric>("1").desc())
                .load(&mut self.connection),
        }
    }

    fn get_chart_granularity_for_period(period: &ChartPeriod) -> ChartGranularity {
        match period {
            ChartPeriod::Hour => ChartGranularity::Minute,
            ChartPeriod::Day => ChartGranularity::Minute15,
            ChartPeriod::Week => ChartGranularity::Hourly,
            ChartPeriod::Month => ChartGranularity::Hour6,
            ChartPeriod::Quarter | ChartPeriod::Year | ChartPeriod::All => ChartGranularity::Daily,
        }
    }

    fn period_sql(&self, period: ChartPeriod) -> &str {
        match period {
            ChartPeriod::Hour => "1 minutes",
            ChartPeriod::Day => "15 minutes",
            ChartPeriod::Week => "1 hour",
            ChartPeriod::Month => "6 hour",
            ChartPeriod::Quarter => "1 day",
            ChartPeriod::Year => "3 day",
            ChartPeriod::All => "3 day",
        }
    }

    fn period_minutes(&self, period: ChartPeriod) -> i32 {
        match period {
            ChartPeriod::Hour => 60,
            ChartPeriod::Day => 1440,
            ChartPeriod::Week => 10_080,
            ChartPeriod::Month => 43_200,
            ChartPeriod::Quarter => 131_400,
            ChartPeriod::Year => 525_600,
            ChartPeriod::All => 10_525_600,
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
