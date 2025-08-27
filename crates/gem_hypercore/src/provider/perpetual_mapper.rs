use crate::models::{
    candlestick::HypercoreCandlestick,
    metadata::HypercoreMetadataResponse,
    position::{HypercoreAssetPositions, HypercoreLeverageType, HypercorePosition},
};
use primitives::{
    chart::ChartCandleStick,
    perpetual::{PerpetualData, PerpetualPositionsSummary},
    {AssetId, Chain, PerpetualBalance, PerpetualDirection, PerpetualMarginType, PerpetualPosition, PerpetualProvider},
};

pub fn map_positions(positions: HypercoreAssetPositions, address: String) -> PerpetualPositionsSummary {
    let balance = map_perpetual_balance(&positions);
    let positions: Vec<PerpetualPosition> = positions
        .asset_positions
        .into_iter()
        .map(|x| map_position(x.position, address.clone()))
        .collect();
    PerpetualPositionsSummary { positions, balance }
}

pub fn map_perpetual_balance(positions: &HypercoreAssetPositions) -> PerpetualBalance {
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
}

pub fn map_position(position: HypercorePosition, address: String) -> PerpetualPosition {
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
        id: format!("{}_{}", address, position.coin.clone()),
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

pub fn map_perpetuals_data(metadata: HypercoreMetadataResponse) -> Vec<PerpetualData> {
    metadata.into()
}

pub fn map_candlesticks(candlesticks: Vec<HypercoreCandlestick>) -> Vec<ChartCandleStick> {
    candlesticks.into_iter().map(|c| c.into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        metadata::{HypercoreAssetMetadata, HypercoreUniverseAsset, HypercoreUniverseResponse},
        position::{
            HypercoreAssetPosition, HypercoreAssetPositions, HypercoreCumulativeFunding, HypercoreLeverage, HypercoreLeverageType, HypercoreMarginSummary,
            HypercorePosition, HypercorePositionType,
        },
    };
    use primitives::{perpetual_provider::PerpetualProvider, PerpetualDirection, PerpetualMarginType};

    #[test]
    fn test_map_positions_basic() {
        let positions = HypercoreAssetPositions {
            asset_positions: vec![HypercoreAssetPosition {
                position_type: HypercorePositionType::OneWay,
                position: HypercorePosition {
                    coin: "BTC".to_string(),
                    szi: "1.5".to_string(),
                    leverage: HypercoreLeverage {
                        leverage_type: HypercoreLeverageType::Cross,
                        value: 10,
                    },
                    entry_px: "50000".to_string(),
                    position_value: "75000".to_string(),
                    unrealized_pnl: "5000".to_string(),
                    return_on_equity: "0.1".to_string(),
                    liquidation_px: Some("40000".to_string()),
                    margin_used: "7500".to_string(),
                    max_leverage: 20,
                    cum_funding: HypercoreCumulativeFunding {
                        all_time: "100".to_string(),
                        since_open: "50".to_string(),
                    },
                },
            }],
            margin_summary: HypercoreMarginSummary {
                account_value: "100000".to_string(),
                total_ntl_pos: "10000".to_string(),
                total_raw_usd: "10000".to_string(),
                total_margin_used: "5000".to_string(),
            },
            cross_margin_summary: HypercoreMarginSummary {
                account_value: "100000".to_string(),
                total_ntl_pos: "10000".to_string(),
                total_raw_usd: "10000".to_string(),
                total_margin_used: "8000".to_string(),
            },
            cross_maintenance_margin_used: "3000".to_string(),
            withdrawable: "92000".to_string(),
        };

        let result = map_positions(positions, "test_address".to_string());

        assert_eq!(result.positions.len(), 1);
        assert_eq!(result.positions[0].id, "test_address_BTC");
        assert_eq!(result.positions[0].size, 1.5);
        assert_eq!(result.positions[0].direction, PerpetualDirection::Long);
        assert_eq!(result.positions[0].margin_type, PerpetualMarginType::Cross);
        assert_eq!(result.positions[0].leverage, 10);
        assert_eq!(result.positions[0].pnl, 5000.0);
        assert_eq!(result.positions[0].funding, Some(-50.0));

        assert_eq!(result.balance.available, 92000.0);
        assert_eq!(result.balance.reserved, 8000.0);
        assert_eq!(result.balance.withdrawable, 92000.0);
    }

    #[test]
    fn test_map_perpetuals_data() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![HypercoreUniverseAsset {
                name: "ETH".to_string(),
                sz_decimals: 4,
                max_leverage: 50,
                only_isolated: Some(false),
            }],
        };

        let asset_metadata = vec![HypercoreAssetMetadata {
            funding: "0.0005".to_string(),
            open_interest: "2500.5".to_string(),
            prev_day_px: "2000".to_string(),
            day_ntl_vlm: "500000".to_string(),
            premium: Some("1.5".to_string()),
            oracle_px: "2100".to_string(),
            mark_px: "2105.25".to_string(),
            mid_px: Some("2102.5".to_string()),
            impact_pxs: Some(vec!["2100".to_string(), "2105".to_string()]),
            day_base_vlm: "250000".to_string(),
        }];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let result = map_perpetuals_data(metadata_response);

        assert_eq!(result.len(), 1);

        let eth_data = &result[0];
        assert_eq!(eth_data.perpetual.id, "hypercore_ETH");
        assert_eq!(eth_data.perpetual.name, "ETH");
        assert_eq!(eth_data.perpetual.provider, PerpetualProvider::Hypercore);
        assert_eq!(eth_data.perpetual.price, 2102.5);
        assert_eq!(eth_data.perpetual.funding, 0.05);
        assert_eq!(eth_data.perpetual.leverage, vec![50]);
        assert_eq!(eth_data.perpetual.volume_24h, 500000.0);

        assert_eq!(eth_data.asset.name, "ETH");
        assert_eq!(eth_data.asset.symbol, "ETH");
        assert_eq!(eth_data.asset.decimals, 4);
        assert_eq!(eth_data.asset.id.to_string(), "hypercore_perpetual::ETH");
    }

    #[test]
    fn test_map_candlesticks() {
        use crate::models::candlestick::HypercoreCandlestick;

        let candlesticks = vec![
            HypercoreCandlestick {
                t: 1640995200000u64, // 2022-01-01 00:00:00 UTC
                o: "50000.0".to_string(),
                h: "51000.0".to_string(),
                l: "49000.0".to_string(),
                c: "50500.0".to_string(),
                v: "100.5".to_string(),
            },
            HypercoreCandlestick {
                t: 1640998800000u64, // 2022-01-01 01:00:00 UTC
                o: "50500.0".to_string(),
                h: "52000.0".to_string(),
                l: "50000.0".to_string(),
                c: "51000.0".to_string(),
                v: "75.2".to_string(),
            },
        ];

        let result = map_candlesticks(candlesticks);

        assert_eq!(result.len(), 2);

        let first_candle = &result[0];
        assert_eq!(first_candle.open, 50000.0);
        assert_eq!(first_candle.high, 51000.0);
        assert_eq!(first_candle.low, 49000.0);
        assert_eq!(first_candle.close, 50500.0);
        assert_eq!(first_candle.volume, 100.5);

        let second_candle = &result[1];
        assert_eq!(second_candle.open, 50500.0);
        assert_eq!(second_candle.high, 52000.0);
        assert_eq!(second_candle.low, 50000.0);
        assert_eq!(second_candle.close, 51000.0);
        assert_eq!(second_candle.volume, 75.2);
    }

    #[test]
    fn test_map_hypercore_positions_to_perpetual_positions_summary() {
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

        let summary = map_positions(positions, "test_user".to_string());

        assert_eq!(summary.positions.len(), 2);

        let sol_position = summary.positions.iter().find(|p| p.id == "test_user_SOL").unwrap();
        assert_eq!(sol_position.size, 10.0);
        assert_eq!(sol_position.size_value, 2029.2);
        assert_eq!(sol_position.leverage, 20);
        assert_eq!(sol_position.margin_type, PerpetualMarginType::Cross);
        assert_eq!(sol_position.direction, PerpetualDirection::Short);
        assert_eq!(sol_position.margin_amount, 101.46);
        assert_eq!(sol_position.pnl, -75.3);
        assert_eq!(sol_position.funding, Some(1.3));

        let btc_position = summary.positions.iter().find(|p| p.id == "test_user_BTC").unwrap();
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
    fn test_map_position_funding_sign_reversal() {
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

        let perpetual_position = map_position(position, "user123".to_string());
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

        let short_perpetual = map_position(short_position, "user123".to_string());
        assert_eq!(short_perpetual.size, 5.0); // Size is always positive (absolute value)
        assert_eq!(short_perpetual.funding, Some(1.5)); // Short position with negative funding
    }

    #[test]
    fn test_map_perpetual_balance() {
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

        let summary = map_positions(positions, "balance_test".to_string());

        assert_eq!(summary.balance.reserved, 1500.25);
        assert_eq!(summary.balance.available, 3500.25);
        assert_eq!(summary.balance.withdrawable, 2500.75);
    }

    #[test]
    fn test_map_perpetual_balance_with_real_data() {
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

        let summary = map_positions(positions, "real_data_test".to_string());

        assert_eq!(summary.balance.reserved, 706.364534);
        assert_eq!(summary.balance.available, 0.0);
        assert_eq!(summary.balance.withdrawable, 305.674569);
    }

    #[test]
    fn test_map_position_asset_id_uses_subtoken_pattern() {
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

        let perpetual_position = map_position(position, "address123".to_string());

        assert_eq!(perpetual_position.asset_id.chain, primitives::Chain::HyperCore);
        assert_eq!(perpetual_position.asset_id.token_id, Some("perpetual::BTC".to_string()));
        assert_eq!(perpetual_position.asset_id.to_string(), "hypercore_perpetual::BTC");
    }
}
