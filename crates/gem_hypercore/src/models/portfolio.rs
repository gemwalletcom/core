use chrono::{DateTime, TimeZone, Utc};
use primitives::portfolio::{Portfolio, PortfolioDataPoint, PortfolioTimeframeData};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercorePortfolioTimeframeData {
    pub account_value_history: Vec<(i64, String)>,
    pub pnl_history: Vec<(i64, String)>,
    pub vlm: String,
}

fn timestamp_to_datetime(timestamp_ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(timestamp_ms).single().unwrap_or_else(Utc::now)
}

fn map_data_points(data: Vec<(i64, String)>) -> Vec<PortfolioDataPoint> {
    data.into_iter()
        .map(|(timestamp, value)| PortfolioDataPoint {
            date: timestamp_to_datetime(timestamp),
            value,
        })
        .collect()
}

impl From<HypercorePortfolioTimeframeData> for PortfolioTimeframeData {
    fn from(data: HypercorePortfolioTimeframeData) -> Self {
        Self {
            account_value_history: map_data_points(data.account_value_history),
            pnl_history: map_data_points(data.pnl_history),
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
