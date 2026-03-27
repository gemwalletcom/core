pub use crate::models::metadata::{AssetMetadata, UniverseAsset};
pub use crate::models::order::OpenOrder;
pub use crate::models::perp_dex::PerpDex;
pub use crate::models::portfolio::{HypercoreDataPoint, HypercorePortfolioResponse, HypercorePortfolioTimeframeData};
pub use crate::models::position::Position;
pub use crate::models::position::{AssetPositions, CumulativeFunding, Leverage, LeverageType, MarginSummary};
#[cfg(test)]
use crate::rpc::client::{HyperCoreClient, InMemoryPreferences};
#[cfg(test)]
use gem_client::testkit::MockClient;
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

impl Position {
    pub fn mock() -> Self {
        Self {
            coin: "BTC".to_string(),
            szi: "1.0".to_string(),
            leverage: Leverage {
                leverage_type: LeverageType::Cross,
                value: 10,
            },
            entry_px: "50000".to_string(),
            position_value: "50000".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "5000".to_string(),
            max_leverage: 10,
            cum_funding: CumulativeFunding {
                all_time: "0".to_string(),
                since_open: "0".to_string(),
            },
        }
    }
}

impl OpenOrder {
    pub fn mock() -> Self {
        Self {
            coin: "HYPE".to_string(),
            oid: 1,
            trigger_px: Some(35.0),
            limit_px: Some(33.5),
            is_position_tpsl: true,
            order_type: "Stop Limit".to_string(),
        }
    }

    pub fn mock_with_trigger(coin: &str, oid: u64, order_type: &str, trigger_px: f64, limit_px: Option<f64>) -> Self {
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
    pub fn mock() -> Self {
        Self {
            account_value_history: vec![HypercoreDataPoint {
                timestamp_ms: 1640995200000,
                value: 1000.0,
            }],
            pnl_history: vec![HypercoreDataPoint {
                timestamp_ms: 1640995200000,
                value: 50.0,
            }],
            vlm: "100".to_string(),
        }
    }

    pub fn mock_with_volume(vlm: &str) -> Self {
        Self {
            vlm: vlm.to_string(),
            ..Self::mock()
        }
    }
}

impl UniverseAsset {
    pub fn mock() -> Self {
        Self {
            name: "ETH".to_string(),
            sz_decimals: 4,
            max_leverage: 50,
            is_isolated_only: None,
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

impl PerpDex {
    pub fn mock() -> Self {
        Self {
            name: "dex".to_string(),
            is_active: Some(true),
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
