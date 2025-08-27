use crate::models::{candlestick::HypercoreCandlestick, metadata::HypercoreMetadataResponse, position::HypercoreAssetPositions};
use primitives::{
    chart::ChartCandleStick,
    perpetual::{PerpetualData, PerpetualPositionsSummary},
};

pub fn map_positions(positions: HypercoreAssetPositions) -> PerpetualPositionsSummary {
    positions.into()
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
    fn test_map_positions() {
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

        let result = map_positions(positions);

        assert_eq!(result.positions.len(), 1);
        assert_eq!(result.positions[0].id, "BTC");
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
}
