use chrono::{DateTime, Utc};

use crate::{
    chart::ChartDateValue,
    portfolio::{PerpetualPortfolio, PerpetualPortfolioTimeframeData},
};

impl PerpetualPortfolioTimeframeData {
    pub fn mock(date: DateTime<Utc>, account_value: f64, pnl: f64, volume: f64) -> Self {
        Self {
            account_value_history: vec![ChartDateValue { date, value: account_value }],
            pnl_history: vec![ChartDateValue { date, value: pnl }],
            volume,
        }
    }
}

impl PerpetualPortfolio {
    pub fn mock_with_day(date: DateTime<Utc>, account_value: f64, pnl: f64, volume: f64) -> Self {
        Self {
            day: Some(PerpetualPortfolioTimeframeData::mock(date, account_value, pnl, volume)),
            ..Default::default()
        }
    }
}
