use chrono::{DateTime, Utc};
use primitives::portfolio::{PerpetualPortfolio, PerpetualPortfolioDataPoint, PerpetualPortfolioTimeframeData};

pub type GemPerpetualPortfolio = PerpetualPortfolio;
pub type GemPerpetualPortfolioTimeframeData = PerpetualPortfolioTimeframeData;
pub type GemPerpetualPortfolioDataPoint = PerpetualPortfolioDataPoint;

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
pub struct GemPerpetualPortfolio {
    pub day: Option<PerpetualPortfolioTimeframeData>,
    pub week: Option<PerpetualPortfolioTimeframeData>,
    pub month: Option<PerpetualPortfolioTimeframeData>,
    pub all_time: Option<PerpetualPortfolioTimeframeData>,
}
