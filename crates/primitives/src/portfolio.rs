use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::chart::ChartDateValue;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, CaseIterable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub enum PerpetualPortfolioChartType {
    Value,
    Pnl,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PerpetualAccountSummary {
    pub account_value: f64,
    pub account_leverage: f64,
    pub margin_usage: f64,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PerpetualPortfolioTimeframeData {
    pub account_value_history: Vec<ChartDateValue>,
    pub pnl_history: Vec<ChartDateValue>,
    pub volume: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PerpetualPortfolio {
    pub day: Option<PerpetualPortfolioTimeframeData>,
    pub week: Option<PerpetualPortfolioTimeframeData>,
    pub month: Option<PerpetualPortfolioTimeframeData>,
    pub all_time: Option<PerpetualPortfolioTimeframeData>,
    pub account_summary: Option<PerpetualAccountSummary>,
}
