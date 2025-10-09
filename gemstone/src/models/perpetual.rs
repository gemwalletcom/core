use chrono::{DateTime, Utc};
use primitives::{
    Asset, AssetId, PerpetualDirection, PerpetualMarginType, PerpetualOrderType, PerpetualPosition, PerpetualProvider, PerpetualTriggerOrder,
    chart::ChartCandleStick,
    perpetual::{Perpetual, PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary},
};

pub type GemPerpetualMarginType = PerpetualMarginType;
pub type GemPerpetualOrderType = PerpetualOrderType;
pub type GemPerpetualPositionsSummary = PerpetualPositionsSummary;
pub type GemPerpetualBalance = PerpetualBalance;
pub type GemPerpetualPosition = PerpetualPosition;
pub type GemPerpetual = Perpetual;
pub type GemPerpetualMetadata = PerpetualMetadata;
pub type GemChartCandleStick = ChartCandleStick;
pub type GemPerpetualData = PerpetualData;

#[uniffi::remote(Enum)]
pub enum GemPerpetualMarginType {
    Cross,
    Isolated,
}

#[uniffi::remote(Enum)]
pub enum GemPerpetualOrderType {
    Market,
    Limit,
}

pub type GemPerpetualTriggerOrder = PerpetualTriggerOrder;

#[uniffi::remote(Record)]
pub struct GemPerpetualTriggerOrder {
    pub price: f64,
    pub order_type: PerpetualOrderType,
    pub order_id: String,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualPositionsSummary {
    pub positions: Vec<PerpetualPosition>,
    pub balance: PerpetualBalance,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualBalance {
    pub available: f64,
    pub reserved: f64,
    pub withdrawable: f64,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualPosition {
    pub id: String,
    pub perpetual_id: String,
    pub asset_id: AssetId,
    pub size: f64,
    pub size_value: f64,
    pub leverage: u8,
    pub entry_price: Option<f64>,
    pub liquidation_price: Option<f64>,
    pub margin_type: PerpetualMarginType,
    pub direction: PerpetualDirection,
    pub margin_amount: f64,
    pub take_profit: Option<PerpetualTriggerOrder>,
    pub stop_loss: Option<PerpetualTriggerOrder>,
    pub pnl: f64,
    pub funding: Option<f32>,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualData {
    pub perpetual: Perpetual,
    pub asset: Asset,
    pub metadata: PerpetualMetadata,
}

#[uniffi::remote(Record)]
pub struct GemPerpetual {
    pub id: String,
    pub name: String,
    pub provider: PerpetualProvider,
    pub asset_id: AssetId,
    pub identifier: String,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub leverage: Vec<u8>,
}

#[uniffi::remote(Record)]
pub struct GemPerpetualMetadata {
    pub is_pinned: bool,
}

#[uniffi::remote(Record)]
pub struct GemChartCandleStick {
    pub date: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
