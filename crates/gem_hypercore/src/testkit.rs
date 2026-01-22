pub use crate::models::order::OpenOrder;
pub use crate::models::portfolio::{HypercoreDataPoint, HypercorePortfolioResponse, HypercorePortfolioTimeframeData};
pub use crate::models::position::{AssetPositions, MarginSummary};

impl AssetPositions {
    pub fn mock() -> Self {
        Self {
            asset_positions: vec![],
            margin_summary: MarginSummary {
                account_value: "10000".to_string(),
                total_ntl_pos: "5000".to_string(),
                total_raw_usd: "5000".to_string(),
                total_margin_used: "2000".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "10000".to_string(),
                total_ntl_pos: "5000".to_string(),
                total_raw_usd: "5000".to_string(),
                total_margin_used: "2000".to_string(),
            },
            cross_maintenance_margin_used: "1000".to_string(),
            withdrawable: "8000".to_string(),
        }
    }
}

impl OpenOrder {
    pub fn mock(coin: &str, oid: u64, order_type: &str, trigger_px: f64, limit_px: Option<f64>) -> Self {
        Self {
            coin: coin.to_string(),
            oid,
            trigger_px: Some(trigger_px),
            limit_px,
            is_position_tpsl: true,
            order_type: order_type.to_string(),
        }
    }
}

impl HypercorePortfolioTimeframeData {
    pub fn mock(vlm: &str) -> Self {
        Self {
            account_value_history: vec![HypercoreDataPoint {
                timestamp_ms: 1640995200000,
                value: 1000.0,
            }],
            pnl_history: vec![HypercoreDataPoint {
                timestamp_ms: 1640995200000,
                value: 50.0,
            }],
            vlm: vlm.to_string(),
        }
    }
}
