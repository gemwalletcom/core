use chrono::{TimeZone, Utc};

use crate::{
    Asset, AssetId, Chain, PerpetualBalance, PerpetualConfirmData, PerpetualDirection, PerpetualMarginType, PerpetualPosition, PerpetualPositionsSummary,
    chart::ChartDateValue,
    portfolio::{PerpetualPortfolio, PerpetualPortfolioTimeframeData},
};

impl PerpetualConfirmData {
    pub fn mock() -> Self {
        Self::mock_with_values(PerpetualDirection::Long, 0, None, None)
    }

    pub fn mock_with_values(direction: PerpetualDirection, asset_index: u32, take_profit: Option<String>, stop_loss: Option<String>) -> Self {
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

impl PerpetualBalance {
    pub fn mock() -> Self {
        Self::mock_with_values(10.0, 5.0, 8.0)
    }

    pub fn mock_with_values(available: f64, reserved: f64, withdrawable: f64) -> Self {
        Self {
            available,
            reserved,
            withdrawable,
        }
    }
}

impl PerpetualPosition {
    pub fn mock() -> Self {
        Self::mock_with_values(
            "one",
            "hypercore_BTC",
            AssetId::from_token(Chain::HyperCore, "perpetual::BTC"),
            PerpetualDirection::Long,
            PerpetualMarginType::Cross,
            1.0,
            100.0,
            5,
            100.0,
            20.0,
            0.0,
        )
    }

    pub fn mock_with_ids(id: &str, perpetual_id: &str, asset_id: AssetId) -> Self {
        Self::mock_with_values(
            id,
            perpetual_id,
            asset_id,
            PerpetualDirection::Long,
            PerpetualMarginType::Cross,
            1.0,
            100.0,
            5,
            100.0,
            20.0,
            0.0,
        )
    }

    pub fn mock_with_values(
        id: &str,
        perpetual_id: &str,
        asset_id: AssetId,
        direction: PerpetualDirection,
        margin_type: PerpetualMarginType,
        size: f64,
        size_value: f64,
        leverage: u8,
        entry_price: f64,
        margin_amount: f64,
        pnl: f64,
    ) -> Self {
        Self {
            id: id.to_string(),
            perpetual_id: perpetual_id.to_string(),
            asset_id,
            size,
            size_value,
            leverage,
            entry_price,
            liquidation_price: None,
            margin_type,
            direction,
            margin_amount,
            take_profit: None,
            stop_loss: None,
            pnl,
            funding: None,
        }
    }
}

impl PerpetualPositionsSummary {
    pub fn mock() -> Self {
        Self {
            positions: vec![PerpetualPosition::mock()],
            balance: PerpetualBalance::mock(),
        }
    }

    pub fn mock_with(positions: Vec<PerpetualPosition>, balance: PerpetualBalance) -> Self {
        Self { positions, balance }
    }
}
