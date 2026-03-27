use chrono::{TimeZone, Utc};

use crate::{
    Asset, Chain, PerpetualConfirmData, PerpetualDirection, PerpetualMarginType,
    chart::ChartDateValue,
    portfolio::{PerpetualPortfolio, PerpetualPortfolioTimeframeData},
};

impl PerpetualConfirmData {
    pub fn mock(direction: PerpetualDirection, asset_index: u32, take_profit: Option<String>, stop_loss: Option<String>) -> Self {
        Self {
            direction,
            margin_type: PerpetualMarginType::Cross,
            base_asset: Asset::from_chain(Chain::HyperCore),
            asset_index: asset_index as i32,
            price: "123.45".to_string(),
            fiat_value: 100.0,
            size: "2.5".to_string(),
            slippage: 0.01,
            leverage: 5,
            pnl: None,
            entry_price: None,
            market_price: 123.45,
            margin_amount: 50.0,
            take_profit,
            stop_loss,
        }
    }
}

impl PerpetualPortfolioTimeframeData {
    pub fn mock() -> Self {
        let date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        Self {
            account_value_history: vec![ChartDateValue { date, value: 1000.0 }],
            pnl_history: vec![ChartDateValue { date, value: 50.0 }],
            volume: 5000.0,
        }
    }
}

impl PerpetualPortfolio {
    pub fn mock() -> Self {
        Self {
            day: Some(PerpetualPortfolioTimeframeData::mock()),
            week: None,
            month: None,
            all_time: None,
            account_summary: None,
        }
    }
}
