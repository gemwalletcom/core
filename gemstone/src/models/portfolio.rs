use chrono::{DateTime, Utc};
use primitives::portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioDataPoint, PerpetualPortfolioTimeframeData};

pub type GemPerpetualPortfolio = PerpetualPortfolio;
pub type GemPerpetualPortfolioTimeframeData = PerpetualPortfolioTimeframeData;
pub type GemPerpetualPortfolioDataPoint = PerpetualPortfolioDataPoint;
pub type GemPerpetualAccountSummary = PerpetualAccountSummary;

#[uniffi::remote(Record)]
pub struct GemPerpetualPortfolioDataPoint {
    pub date: DateTime<Utc>,
    pub value: f64,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualPortfolioTimeframeData {
    pub account_value_history: Vec<PerpetualPortfolioDataPoint>,
    pub pnl_history: Vec<PerpetualPortfolioDataPoint>,
    pub volume: f64,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualAccountSummary {
    pub account_value: f64,
    pub account_leverage: f64,
    pub margin_usage: f64,
    pub unrealized_pnl: f64,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualPortfolio {
    pub day: Option<PerpetualPortfolioTimeframeData>,
    pub week: Option<PerpetualPortfolioTimeframeData>,
    pub month: Option<PerpetualPortfolioTimeframeData>,
    pub all_time: Option<PerpetualPortfolioTimeframeData>,
    pub account_summary: Option<PerpetualAccountSummary>,
}
