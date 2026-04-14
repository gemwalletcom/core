use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::asset_id::AssetId;
use crate::asset_price::{ChartPeriod, ChartValue};
use crate::chart::ChartDateValue;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, CaseIterable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub enum PortfolioType {
    Wallet,
    Perpetuals,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, CaseIterable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub enum PortfolioChartType {
    Value,
    Pnl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioChartData {
    pub chart_type: PortfolioChartType,
    pub values: Vec<ChartDateValue>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAsset {
    pub asset_id: AssetId,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAssetsRequest {
    pub assets: Vec<PortfolioAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAllocation {
    pub asset_id: AssetId,
    pub percentage: f32,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct ChartValuePercentage {
    pub date: DateTime<Utc>,
    pub value: f32,
    pub percentage: f32,
}

impl ChartValuePercentage {
    pub fn with_rate(&self, rate: f64) -> Self {
        Self {
            date: self.date,
            value: self.value * rate as f32,
            percentage: self.percentage,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAssets {
    pub total_value: f32,
    pub values: Vec<ChartValue>,
    pub all_time_high: Option<ChartValuePercentage>,
    pub all_time_low: Option<ChartValuePercentage>,
    pub allocation: Vec<PortfolioAllocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioMarginUsage {
    pub account_value: f64,
    pub usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PortfolioStatistic {
    AllTimeHigh(ChartValuePercentage),
    AllTimeLow(ChartValuePercentage),
    UnrealizedPnl(f64),
    AccountLeverage(f64),
    MarginUsage(PortfolioMarginUsage),
    AllTimePnl(f64),
    Volume(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PortfolioData {
    pub charts: Vec<PortfolioChartData>,
    pub statistics: Vec<PortfolioStatistic>,
    pub available_periods: Vec<ChartPeriod>,
}
