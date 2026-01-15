use chrono::{DateTime, Utc};
use primitives::portfolio::{Portfolio, PortfolioDataPoint, PortfolioTimeframeData};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "(i64, String)")]
pub struct HypercoreDataPoint {
    pub timestamp_ms: i64,
    pub value: String,
}

impl From<(i64, String)> for HypercoreDataPoint {
    fn from((timestamp_ms, value): (i64, String)) -> Self {
        Self { timestamp_ms, value }
    }
}

impl From<HypercoreDataPoint> for PortfolioDataPoint {
    fn from(point: HypercoreDataPoint) -> Self {
        Self {
            date: DateTime::from_timestamp_millis(point.timestamp_ms).unwrap_or_else(Utc::now),
            value: point.value,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercorePortfolioTimeframeData {
    pub account_value_history: Vec<HypercoreDataPoint>,
    pub pnl_history: Vec<HypercoreDataPoint>,
    pub vlm: String,
}

impl From<HypercorePortfolioTimeframeData> for PortfolioTimeframeData {
    fn from(data: HypercorePortfolioTimeframeData) -> Self {
        Self {
            account_value_history: data.account_value_history.into_iter().map(Into::into).collect(),
            pnl_history: data.pnl_history.into_iter().map(Into::into).collect(),
            volume: data.vlm,
        }
    }
}

pub fn parse_portfolio_response(raw: Vec<(String, HypercorePortfolioTimeframeData)>) -> Portfolio {
    let mut portfolio = Portfolio::default();
    for (timeframe, data) in raw {
        match timeframe.as_str() {
            "day" => portfolio.day = Some(data.into()),
            "week" => portfolio.week = Some(data.into()),
            "month" => portfolio.month = Some(data.into()),
            "allTime" => portfolio.all_time = Some(data.into()),
            "perpDay" => portfolio.perp_day = Some(data.into()),
            "perpWeek" => portfolio.perp_week = Some(data.into()),
            "perpMonth" => portfolio.perp_month = Some(data.into()),
            "perpAllTime" => portfolio.perp_all_time = Some(data.into()),
            _ => {}
        }
    }
    portfolio
}
