use primitives::chart::ChartCandleStick;
use primitives::perpetual::{Perpetual, PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary};
use primitives::PerpetualPosition;

use super::asset::GemAsset;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualPositionsSummary {
    pub positions: Vec<GemPerpetualPosition>,
    pub balance: GemPerpetualBalance,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualBalance {
    pub available: f64,
    pub reserved: f64,
    pub withdrawable: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualPosition {
    pub asset_id: String,
    pub perpetual_id: String,
    pub symbol: String,
    pub direction: String,
    pub size: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
    pub margin: f64,
    pub leverage: f64,
    pub entry_price: f64,
    pub liquidation_price: Option<f64>,
    pub funding: Option<f32>,
    pub margin_type: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualData {
    pub perpetual: GemPerpetual,
    pub asset: GemAsset,
    pub metadata: GemPerpetualMetadata,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetual {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub asset_id: String,
    pub identifier: String,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub leverage: Vec<u8>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualMetadata {
    pub is_pinned: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemChartCandleStick {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl From<PerpetualPositionsSummary> for GemPerpetualPositionsSummary {
    fn from(summary: PerpetualPositionsSummary) -> Self {
        Self {
            positions: summary.positions.into_iter().map(|p| p.into()).collect(),
            balance: summary.balance.into(),
        }
    }
}

impl From<PerpetualBalance> for GemPerpetualBalance {
    fn from(balance: PerpetualBalance) -> Self {
        Self {
            available: balance.available,
            reserved: balance.reserved,
            withdrawable: balance.withdrawable,
        }
    }
}

impl From<PerpetualPosition> for GemPerpetualPosition {
    fn from(position: PerpetualPosition) -> Self {
        Self {
            asset_id: position.asset_id.to_string(),
            perpetual_id: position.perpetual_id,
            symbol: position.id,
            direction: position.direction.as_ref().to_string(),
            size: position.size,
            pnl: position.pnl,
            pnl_percent: {
                let entry_price = position.entry_price.unwrap_or(0.0);
                if entry_price > 0.0 {
                    (position.pnl / (position.size * entry_price)) * 100.0
                } else {
                    0.0
                }
            },
            margin: position.margin_amount,
            leverage: position.leverage as f64,
            entry_price: position.entry_price.unwrap_or(0.0),
            liquidation_price: position.liquidation_price,
            funding: position.funding,
            margin_type: position.margin_type.as_ref().to_string(),
        }
    }
}

impl From<PerpetualData> for GemPerpetualData {
    fn from(data: PerpetualData) -> Self {
        Self {
            perpetual: data.perpetual.into(),
            asset: data.asset.into(),
            metadata: data.metadata.into(),
        }
    }
}

impl From<Perpetual> for GemPerpetual {
    fn from(perpetual: Perpetual) -> Self {
        Self {
            id: perpetual.id,
            name: perpetual.name,
            provider: perpetual.provider.as_ref().to_string(),
            asset_id: perpetual.asset_id.to_string(),
            identifier: perpetual.identifier,
            price: perpetual.price,
            price_percent_change_24h: perpetual.price_percent_change_24h,
            open_interest: perpetual.open_interest,
            volume_24h: perpetual.volume_24h,
            funding: perpetual.funding,
            leverage: perpetual.leverage,
        }
    }
}

impl From<PerpetualMetadata> for GemPerpetualMetadata {
    fn from(metadata: PerpetualMetadata) -> Self {
        Self { is_pinned: metadata.is_pinned }
    }
}

impl From<ChartCandleStick> for GemChartCandleStick {
    fn from(candlestick: ChartCandleStick) -> Self {
        Self {
            timestamp: candlestick.date.timestamp(),
            open: candlestick.open,
            high: candlestick.high,
            low: candlestick.low,
            close: candlestick.close,
            volume: candlestick.volume,
        }
    }
}
