use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioDataPoint {
    pub date: DateTime<Utc>,
    pub value: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioTimeframeData {
    pub account_value_history: Vec<PortfolioDataPoint>,
    pub pnl_history: Vec<PortfolioDataPoint>,
    pub volume: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct Portfolio {
    pub day: Option<PortfolioTimeframeData>,
    pub week: Option<PortfolioTimeframeData>,
    pub month: Option<PortfolioTimeframeData>,
    pub all_time: Option<PortfolioTimeframeData>,
    pub perp_day: Option<PortfolioTimeframeData>,
    pub perp_week: Option<PortfolioTimeframeData>,
    pub perp_month: Option<PortfolioTimeframeData>,
    pub perp_all_time: Option<PortfolioTimeframeData>,
}
