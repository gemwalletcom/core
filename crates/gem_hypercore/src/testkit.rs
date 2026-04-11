pub use crate::models::metadata::{AssetMetadata, UniverseAsset};
pub use crate::models::order::OpenOrder;
pub use crate::models::portfolio::{HypercoreDataPoint, HypercorePortfolioResponse, HypercorePortfolioTimeframeData};
pub use crate::models::position::{AssetPositions, MarginSummary};
#[cfg(test)]
use crate::rpc::client::HyperCoreClient;
#[cfg(test)]
use gem_client::testkit::MockClient;
#[cfg(test)]
use primitives::InMemoryPreferences;
#[cfg(test)]
use std::sync::Arc;

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

impl UniverseAsset {
    pub fn mock() -> Self {
        Self {
            name: "ETH".to_string(),
            sz_decimals: 4,
            max_leverage: 50,
            only_isolated: None,
        }
    }
}

impl AssetMetadata {
    pub fn mock() -> Self {
        Self {
            funding: "0.0005".to_string(),
            open_interest: "2500.5".to_string(),
            prev_day_px: "2000".to_string(),
            day_ntl_vlm: "500000".to_string(),
            premium: None,
            oracle_px: "2100".to_string(),
            mark_px: "2105.25".to_string(),
            mid_px: Some("2102.5".to_string()),
            impact_pxs: None,
            day_base_vlm: "250000".to_string(),
        }
    }
}

#[cfg(test)]
impl HyperCoreClient<MockClient> {
    pub fn mock() -> Self {
        let preferences = Arc::new(InMemoryPreferences::new());
        let secure_preferences = Arc::new(InMemoryPreferences::new());
        Self::new_with_preferences(MockClient::new(), preferences, secure_preferences)
    }
}
