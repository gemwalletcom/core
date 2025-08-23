use primitives::{AssetId, Chain, PerpetualBalance, PerpetualDirection, PerpetualMarginType, PerpetualPosition, PerpetualPositionsSummary, PerpetualProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreAssetPositions {
    pub asset_positions: Vec<HypercoreAssetPosition>,
    pub margin_summary: HypercoreMarginSummary,
    pub cross_margin_summary: HypercoreMarginSummary,
    pub cross_maintenance_margin_used: String,
    pub withdrawable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreMarginSummary {
    pub account_value: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
    pub total_margin_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreAssetPosition {
    #[serde(rename = "type")]
    pub position_type: HypercorePositionType,
    pub position: HypercorePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HypercorePositionType {
    OneWay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercorePosition {
    pub coin: String,
    pub szi: String,
    pub leverage: HypercoreLeverage,
    pub entry_px: String,
    pub position_value: String,
    pub unrealized_pnl: String,
    pub return_on_equity: String,
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub max_leverage: u32,
    pub cum_funding: HypercoreCumulativeFunding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreLeverage {
    #[serde(rename = "type")]
    pub leverage_type: HypercoreLeverageType,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HypercoreLeverageType {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreCumulativeFunding {
    pub all_time: String,
    pub since_open: String,
}

impl From<HypercoreAssetPositions> for PerpetualPositionsSummary {
    fn from(positions: HypercoreAssetPositions) -> Self {
        let balance = {
            let equity = positions.margin_summary.account_value.parse().unwrap_or(0.0);
            let margin_used = positions.cross_margin_summary.total_margin_used.parse().unwrap_or(0.0);
            let reserved = f64::min(f64::max(margin_used, 0.0), f64::max(equity, 0.0));
            let available = f64::max(equity - reserved, 0.0);
            let withdrawable = positions.withdrawable.parse().unwrap_or(0.0);

            PerpetualBalance {
                available,
                reserved,
                withdrawable,
            }
        };

        let positions: Vec<PerpetualPosition> = positions.asset_positions.into_iter().map(|x| x.position.into()).collect();

        PerpetualPositionsSummary { positions, balance }
    }
}

impl From<HypercorePosition> for PerpetualPosition {
    fn from(position: HypercorePosition) -> Self {
        let size: f64 = position.szi.parse().unwrap_or(0.0);
        let direction = if size >= 0.0 { PerpetualDirection::Long } else { PerpetualDirection::Short };

        let raw_funding = position.cum_funding.since_open.parse::<f32>().unwrap_or(0.0);
        let funding_value = match direction {
            PerpetualDirection::Long => Some(-raw_funding),
            PerpetualDirection::Short => {
                if raw_funding < 0.0 {
                    Some(-raw_funding)
                } else {
                    Some(raw_funding)
                }
            }
        };
        let perpetual_id = format!("{}_{}", PerpetualProvider::Hypercore.as_ref(), position.coin.clone());
        let asset_id = AssetId::from(Chain::HyperCore, Some(AssetId::sub_token_id(&["perpetual".to_string(), position.coin.clone()])));

        PerpetualPosition {
            id: position.coin.clone(),
            perpetual_id,
            asset_id,
            size: size.abs(),
            size_value: position.position_value.parse::<f64>().unwrap_or(0.0).abs(),
            leverage: position.leverage.value as u8,
            entry_price: Some(position.entry_px.parse().unwrap_or(0.0)),
            liquidation_price: position.liquidation_px.and_then(|p| p.parse().ok()),
            margin_type: match position.leverage.leverage_type {
                HypercoreLeverageType::Cross => PerpetualMarginType::Cross,
                HypercoreLeverageType::Isolated => PerpetualMarginType::Isolated,
            },
            direction,
            margin_amount: position.margin_used.parse().unwrap_or(0.0),
            take_profit: None,
            stop_loss: None,
            pnl: position.unrealized_pnl.parse().unwrap_or(0.0),
            funding: funding_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::PerpetualDirection;
    use primitives::PerpetualMarginType;

    #[test]
    fn test_hypercore_positions_to_perpetual_positions_summary() {
        let positions = HypercoreAssetPositions {
            asset_positions: vec![
                HypercoreAssetPosition {
                    position_type: HypercorePositionType::OneWay,
                    position: HypercorePosition {
                        coin: "SOL".to_string(),
                        szi: "-10.0".to_string(),
                        leverage: HypercoreLeverage {
                            leverage_type: HypercoreLeverageType::Cross,
                            value: 20,
                        },
                        entry_px: "195.39".to_string(),
                        position_value: "2029.2".to_string(),
                        unrealized_pnl: "-75.3".to_string(),
                        return_on_equity: "-0.77076616".to_string(),
                        liquidation_px: Some("558.9517436098".to_string()),
                        margin_used: "101.46".to_string(),
                        max_leverage: 20,
                        cum_funding: HypercoreCumulativeFunding {
                            all_time: "-1.3358".to_string(),
                            since_open: "-1.3".to_string(),
                        },
                    },
                },
                HypercoreAssetPosition {
                    position_type: HypercorePositionType::OneWay,
                    position: HypercorePosition {
                        coin: "BTC".to_string(),
                        szi: "3.0".to_string(),
                        leverage: HypercoreLeverage {
                            leverage_type: HypercoreLeverageType::Isolated,
                            value: 10,
                        },
                        entry_px: "766.34".to_string(),
                        position_value: "2332.2".to_string(),
                        unrealized_pnl: "33.18".to_string(),
                        return_on_equity: "0.1443223634".to_string(),
                        liquidation_px: None,
                        margin_used: "233.22".to_string(),
                        max_leverage: 10,
                        cum_funding: HypercoreCumulativeFunding {
                            all_time: "1.686397".to_string(),
                            since_open: "1.1".to_string(),
                        },
                    },
                },
            ],
            margin_summary: HypercoreMarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_margin_summary: HypercoreMarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_maintenance_margin_used: "50".to_string(),
            withdrawable: "500".to_string(),
        };

        let summary: PerpetualPositionsSummary = positions.into();

        assert_eq!(summary.positions.len(), 2);

        let sol_position = summary.positions.iter().find(|p| p.id == "SOL").unwrap();
        assert_eq!(sol_position.size, 10.0);
        assert_eq!(sol_position.size_value, 2029.2);
        assert_eq!(sol_position.leverage, 20);
        assert_eq!(sol_position.margin_type, PerpetualMarginType::Cross);
        assert_eq!(sol_position.direction, PerpetualDirection::Short);
        assert_eq!(sol_position.margin_amount, 101.46);
        assert_eq!(sol_position.pnl, -75.3);
        assert_eq!(sol_position.funding, Some(1.3));

        let btc_position = summary.positions.iter().find(|p| p.id == "BTC").unwrap();
        assert_eq!(btc_position.size, 3.0);
        assert_eq!(btc_position.size_value, 2332.2);
        assert_eq!(btc_position.leverage, 10);
        assert_eq!(btc_position.margin_type, PerpetualMarginType::Isolated);
        assert_eq!(btc_position.direction, PerpetualDirection::Long);
        assert_eq!(btc_position.margin_amount, 233.22);
        assert_eq!(btc_position.pnl, 33.18);
        assert_eq!(btc_position.funding, Some(-1.1));
    }

    #[test]
    fn test_funding_sign_reversal() {
        let position = HypercorePosition {
            coin: "BTC".to_string(),
            szi: "3.0".to_string(), // Long position
            leverage: HypercoreLeverage {
                leverage_type: HypercoreLeverageType::Cross,
                value: 10,
            },
            entry_px: "100".to_string(),
            position_value: "300".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "30".to_string(),
            max_leverage: 10,
            cum_funding: HypercoreCumulativeFunding {
                all_time: "1.5".to_string(),
                since_open: "1.5".to_string(),
            },
        };

        let perpetual_position: PerpetualPosition = position.into();
        assert_eq!(perpetual_position.funding, Some(-1.5)); // Long position reverses sign

        let short_position = HypercorePosition {
            coin: "ETH".to_string(),
            szi: "-5.0".to_string(), // Short position
            leverage: HypercoreLeverage {
                leverage_type: HypercoreLeverageType::Cross,
                value: 10,
            },
            entry_px: "100".to_string(),
            position_value: "500".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "50".to_string(),
            max_leverage: 10,
            cum_funding: HypercoreCumulativeFunding {
                all_time: "-1.5".to_string(),
                since_open: "-1.5".to_string(),
            },
        };

        let short_perpetual: PerpetualPosition = short_position.into();
        assert_eq!(short_perpetual.size, 5.0); // Size is always positive (absolute value)
        assert_eq!(short_perpetual.funding, Some(1.5)); // Short position with negative funding
    }

    #[test]
    fn test_perpetual_balance_mapping() {
        let positions = HypercoreAssetPositions {
            asset_positions: vec![],
            margin_summary: HypercoreMarginSummary {
                account_value: "5000.50".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_margin_summary: HypercoreMarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "1500.25".to_string(),
            },
            cross_maintenance_margin_used: "50".to_string(),
            withdrawable: "2500.75".to_string(),
        };

        let summary: PerpetualPositionsSummary = positions.into();

        assert_eq!(summary.balance.reserved, 1500.25);
        assert_eq!(summary.balance.available, 3500.25);
        assert_eq!(summary.balance.withdrawable, 2500.75);
    }

    #[test]
    fn test_perpetual_balance_with_real_data() {
        let positions = HypercoreAssetPositions {
            asset_positions: vec![],
            margin_summary: HypercoreMarginSummary {
                account_value: "706.364534".to_string(),
                total_ntl_pos: "12013.47849".to_string(),
                total_raw_usd: "2737.835324".to_string(),
                total_margin_used: "926.155026".to_string(),
            },
            cross_margin_summary: HypercoreMarginSummary {
                account_value: "706.364534".to_string(),
                total_ntl_pos: "12013.47849".to_string(),
                total_raw_usd: "2737.835324".to_string(),
                total_margin_used: "926.155026".to_string(),
            },
            cross_maintenance_margin_used: "400.689965".to_string(),
            withdrawable: "305.674569".to_string(),
        };

        let summary: PerpetualPositionsSummary = positions.into();

        assert_eq!(summary.balance.reserved, 706.364534);
        assert_eq!(summary.balance.available, 0.0);
        assert_eq!(summary.balance.withdrawable, 305.674569);
    }

    #[test]
    fn test_asset_id_uses_subtoken_pattern() {
        let position = HypercorePosition {
            coin: "BTC".to_string(),
            szi: "1.0".to_string(),
            leverage: HypercoreLeverage {
                leverage_type: HypercoreLeverageType::Cross,
                value: 10,
            },
            entry_px: "50000".to_string(),
            position_value: "50000".to_string(),
            unrealized_pnl: "0".to_string(),
            return_on_equity: "0".to_string(),
            liquidation_px: None,
            margin_used: "5000".to_string(),
            max_leverage: 10,
            cum_funding: HypercoreCumulativeFunding {
                all_time: "0".to_string(),
                since_open: "0".to_string(),
            },
        };

        let perpetual_position: PerpetualPosition = position.into();

        assert_eq!(perpetual_position.asset_id.chain, primitives::Chain::HyperCore);
        assert_eq!(perpetual_position.asset_id.token_id, Some("perpetual::BTC".to_string()));
        assert_eq!(perpetual_position.asset_id.to_string(), "hypercore_perpetual::BTC");
    }
}
