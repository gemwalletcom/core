use chrono::{DateTime, Utc};
use primitives::portfolio::{Portfolio, PortfolioDataPoint, PortfolioTimeframeData};

pub type GemPortfolio = Portfolio;
pub type GemPortfolioTimeframeData = PortfolioTimeframeData;
pub type GemPortfolioDataPoint = PortfolioDataPoint;

#[uniffi::remote(Record)]
pub struct GemPortfolioDataPoint {
    pub date: DateTime<Utc>,
    pub value: String,
}

#[uniffi::remote(Record)]
pub struct GemPortfolioTimeframeData {
    pub account_value_history: Vec<PortfolioDataPoint>,
    pub pnl_history: Vec<PortfolioDataPoint>,
    pub volume: String,
}

#[uniffi::remote(Record)]
pub struct GemPortfolio {
    pub day: Option<PortfolioTimeframeData>,
    pub week: Option<PortfolioTimeframeData>,
    pub month: Option<PortfolioTimeframeData>,
    pub all_time: Option<PortfolioTimeframeData>,
    pub perp_day: Option<PortfolioTimeframeData>,
    pub perp_week: Option<PortfolioTimeframeData>,
    pub perp_month: Option<PortfolioTimeframeData>,
    pub perp_all_time: Option<PortfolioTimeframeData>,
}
