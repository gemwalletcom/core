use std::collections::BTreeMap;

use gem_evm::ethereum_address_checksum;
use primitives::{
    Asset, AssetId, AssetType, Chain, Perpetual, PerpetualBalance, PerpetualDirection, PerpetualMarginType, PerpetualOrderType, PerpetualPosition, PerpetualProvider,
    PerpetualTriggerOrder,
    chart::{ChartCandleStick, ChartDateValue},
    perpetual::{PerpetualData, PerpetualMetadata, PerpetualPositionsSummary},
    portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData},
};

use crate::models::{
    candlestick::Candlestick,
    metadata::HypercoreMetadataResponse,
    order::OpenOrder,
    portfolio::HypercorePortfolioResponse,
    position::{AssetPositions, LeverageType, Position},
};

const HIP3_PERP_ASSET_OFFSET: u32 = 100_000;
const HIP3_PERP_ASSET_STRIDE: u32 = 10_000;

pub fn create_perpetual_asset_id(coin: &str) -> AssetId {
    crate::models::metadata::perpetual_asset_id(coin)
}

pub fn create_perpetual_asset_id_by_dex(coin: &str, perp_dex_index: u32) -> AssetId {
    crate::models::metadata::perpetual_asset_id_by_dex(coin, perp_dex_index)
}

pub fn create_perpetual_id(coin: &str) -> String {
    format!("{}_{}", PerpetualProvider::Hypercore.as_ref(), coin)
}

pub fn create_perpetual_id_by_dex(coin: &str, perp_dex_index: u32) -> String {
    if perp_dex_index == 0 {
        return create_perpetual_id(coin);
    }

    format!("{}_{}_{}", PerpetualProvider::Hypercore.as_ref(), perp_dex_index, coin)
}

pub fn map_positions(positions: AssetPositions, address: &str, orders: &[OpenOrder], perp_dex_index: u32) -> PerpetualPositionsSummary {
    let balance = map_perpetual_balance(&positions);
    let positions: Vec<PerpetualPosition> = positions
        .asset_positions
        .into_iter()
        .map(|x| map_position(x.position, address, orders, perp_dex_index))
        .collect();
    PerpetualPositionsSummary { positions, balance }
}

pub fn map_perpetual_balance(positions: &AssetPositions) -> PerpetualBalance {
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

pub fn map_position(position: Position, address: &str, orders: &[OpenOrder], perp_dex_index: u32) -> PerpetualPosition {
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
    let perpetual_id = create_perpetual_id_by_dex(&position.coin, perp_dex_index);
    let asset_id = create_perpetual_asset_id_by_dex(&position.coin, perp_dex_index);
    let address = ethereum_address_checksum(address).unwrap_or(address.to_string());

    let (take_profit, stop_loss) = map_tp_sl_from_orders(orders, &position.coin);
    let id = match perp_dex_index {
        0 => format!("{}_{}", address, position.coin),
        _ => format!("{}_{}_{}", address, perp_dex_index, position.coin),
    };

    PerpetualPosition {
        id,
        perpetual_id,
        asset_id,
        size: size.abs(),
        size_value: position.position_value.parse::<f64>().unwrap_or(0.0).abs(),
        leverage: position.leverage.value as u8,
        entry_price: position.entry_px.parse().unwrap_or(0.0),
        liquidation_price: position.liquidation_px.and_then(|p| p.parse().ok()),
        margin_type: match position.leverage.leverage_type {
            LeverageType::Cross => PerpetualMarginType::Cross,
            LeverageType::Isolated => PerpetualMarginType::Isolated,
        },
        direction,
        margin_amount: position.margin_used.parse().unwrap_or(0.0),
        take_profit,
        stop_loss,
        pnl: position.unrealized_pnl.parse().unwrap_or(0.0),
        funding: funding_value,
    }
}

pub fn map_perpetuals_data(metadata: HypercoreMetadataResponse, perp_dex_index: u32) -> Vec<PerpetualData> {
    let universe = metadata.universe();
    let asset_metadata = metadata.asset_metadata();

    universe
        .universe
        .iter()
        .enumerate()
        .map(|(index, universe_asset)| {
            let metadata_item = asset_metadata.get(index);

            let asset_index = perp_asset_index(perp_dex_index, index as u32);
            let asset_id = universe_asset.asset_id_by_dex(perp_dex_index);

            let current_price = metadata_item
                .and_then(|m| m.mid_px.as_ref().and_then(|mid| mid.parse().ok()).or_else(|| m.mark_px.parse().ok()))
                .unwrap_or(0.0);

            let prev_price = metadata_item.and_then(|m| m.prev_day_px.parse().ok()).unwrap_or(0.0);

            let price_change_24h = if prev_price > 0.0 { ((current_price - prev_price) / prev_price) * 100.0 } else { 0.0 };

            let funding_rate = metadata_item.and_then(|m| m.funding.parse::<f64>().ok()).unwrap_or(0.0) * 100.0;

            let open_interest_coins = metadata_item.and_then(|m| m.open_interest.parse::<f64>().ok()).unwrap_or(0.0);
            let open_interest_usd = open_interest_coins * current_price;

            let perpetual_id = create_perpetual_id_by_dex(&universe_asset.name, perp_dex_index);
            let perpetual = Perpetual {
                id: perpetual_id,
                name: universe_asset.name.clone(),
                provider: PerpetualProvider::Hypercore,
                asset_id: asset_id.clone(),
                identifier: asset_index.to_string(),
                price: current_price,
                price_percent_change_24h: price_change_24h,
                open_interest: open_interest_usd,
                volume_24h: metadata_item.and_then(|m| m.day_ntl_vlm.parse().ok()).unwrap_or(0.0),
                funding: funding_rate,
                max_leverage: universe_asset.max_leverage as u8,
                is_isolated_only: universe_asset.is_isolated_only.unwrap_or(false),
            };

            let asset = Asset {
                id: asset_id,
                chain: Chain::HyperCore,
                token_id: Some(universe_asset.name.clone()),
                name: universe_asset.name.clone(),
                symbol: universe_asset.name.clone(),
                decimals: universe_asset.sz_decimals,
                asset_type: AssetType::PERPETUAL,
            };

            let metadata = PerpetualMetadata { is_pinned: false };

            PerpetualData { perpetual, asset, metadata }
        })
        .collect()
}

pub fn map_candlesticks(candlesticks: Vec<Candlestick>) -> Vec<ChartCandleStick> {
    candlesticks.iter().map(ChartCandleStick::from).collect()
}

pub fn map_account_summary(positions: &AssetPositions) -> PerpetualAccountSummary {
    let account_value = positions.margin_summary.account_value.parse::<f64>().unwrap_or(0.0);
    let total_ntl_pos = positions.margin_summary.total_ntl_pos.parse::<f64>().unwrap_or(0.0);
    let total_margin_used = positions.margin_summary.total_margin_used.parse::<f64>().unwrap_or(0.0);

    let account_leverage = if account_value > 0.0 { total_ntl_pos / account_value } else { 0.0 };
    let margin_usage = if account_value > 0.0 { total_margin_used / account_value } else { 0.0 };

    let unrealized_pnl: f64 = positions.asset_positions.iter().map(|p| p.position.unrealized_pnl.parse::<f64>().unwrap_or(0.0)).sum();

    PerpetualAccountSummary {
        account_value,
        account_leverage,
        margin_usage,
        unrealized_pnl,
    }
}

pub fn map_perpetual_portfolio(response: HypercorePortfolioResponse, positions: &AssetPositions) -> PerpetualPortfolio {
    let (day, week, month, all_time) = response
        .timeframes
        .into_iter()
        .fold((None, None, None, None), |(day, week, month, all_time), (timeframe, data)| match timeframe.as_str() {
            "perpDay" => (Some(data.into()), week, month, all_time),
            "perpWeek" => (day, Some(data.into()), month, all_time),
            "perpMonth" => (day, week, Some(data.into()), all_time),
            "perpAllTime" => (day, week, month, Some(data.into())),
            _ => (day, week, month, all_time),
        });

    PerpetualPortfolio {
        day,
        week,
        month,
        all_time,
        account_summary: Some(map_account_summary(positions)),
    }
}

fn perp_asset_index(perp_dex_index: u32, meta_index: u32) -> u32 {
    if perp_dex_index == 0 {
        meta_index
    } else {
        HIP3_PERP_ASSET_OFFSET + perp_dex_index * HIP3_PERP_ASSET_STRIDE + meta_index
    }
}

pub fn map_account_summary_aggregate(positions: &[AssetPositions]) -> PerpetualAccountSummary {
    let account_value: f64 = positions.iter().map(|p| p.margin_summary.account_value.parse().unwrap_or(0.0)).sum();
    let total_ntl_pos: f64 = positions.iter().map(|p| p.margin_summary.total_ntl_pos.parse().unwrap_or(0.0)).sum();
    let total_margin_used: f64 = positions.iter().map(|p| p.margin_summary.total_margin_used.parse().unwrap_or(0.0)).sum();
    let unrealized_pnl: f64 = positions
        .iter()
        .flat_map(|p| &p.asset_positions)
        .map(|p| p.position.unrealized_pnl.parse().unwrap_or(0.0))
        .sum();

    let account_leverage = if account_value > 0.0 { total_ntl_pos / account_value } else { 0.0 };
    let margin_usage = if account_value > 0.0 { total_margin_used / account_value } else { 0.0 };

    PerpetualAccountSummary {
        account_value,
        account_leverage,
        margin_usage,
        unrealized_pnl,
    }
}

pub fn merge_positions_summaries(summaries: Vec<PerpetualPositionsSummary>) -> PerpetualPositionsSummary {
    summaries.into_iter().fold(
        PerpetualPositionsSummary {
            positions: Vec::new(),
            balance: PerpetualBalance {
                available: 0.0,
                reserved: 0.0,
                withdrawable: 0.0,
            },
        },
        |mut aggregate, summary| {
            aggregate.positions.extend(summary.positions);
            aggregate.balance.available += summary.balance.available;
            aggregate.balance.reserved += summary.balance.reserved;
            aggregate.balance.withdrawable += summary.balance.withdrawable;
            aggregate
        },
    )
}

pub fn merge_perpetual_portfolios(portfolios: Vec<PerpetualPortfolio>, account_summary: Option<PerpetualAccountSummary>) -> PerpetualPortfolio {
    let mut day = Vec::new();
    let mut week = Vec::new();
    let mut month = Vec::new();
    let mut all_time = Vec::new();

    for portfolio in portfolios {
        day.extend(portfolio.day);
        week.extend(portfolio.week);
        month.extend(portfolio.month);
        all_time.extend(portfolio.all_time);
    }

    PerpetualPortfolio {
        day: merge_portfolio_timeframes(day),
        week: merge_portfolio_timeframes(week),
        month: merge_portfolio_timeframes(month),
        all_time: merge_portfolio_timeframes(all_time),
        account_summary,
    }
}

fn merge_portfolio_timeframes(values: Vec<PerpetualPortfolioTimeframeData>) -> Option<PerpetualPortfolioTimeframeData> {
    if values.is_empty() {
        return None;
    }

    let volume: f64 = values.iter().map(|v| v.volume).sum();
    let (account_value_histories, pnl_histories): (Vec<_>, Vec<_>) = values.into_iter().map(|v| (v.account_value_history, v.pnl_history)).unzip();

    Some(PerpetualPortfolioTimeframeData {
        account_value_history: merge_chart_histories(account_value_histories),
        pnl_history: merge_chart_histories(pnl_histories),
        volume,
    })
}

fn merge_chart_histories(values: Vec<Vec<ChartDateValue>>) -> Vec<ChartDateValue> {
    let mut grouped = BTreeMap::new();
    for history in values {
        for point in history {
            let entry = grouped.entry(point.date).or_insert(0.0);
            *entry += point.value;
        }
    }

    grouped.into_iter().map(|(date, value)| ChartDateValue { date, value }).collect()
}

fn determine_order_type(order_type_str: &str) -> PerpetualOrderType {
    if order_type_str.to_lowercase().contains("market") {
        PerpetualOrderType::Market
    } else {
        PerpetualOrderType::Limit
    }
}

pub fn map_tp_sl_from_orders(orders: &[OpenOrder], coin: &str) -> (Option<PerpetualTriggerOrder>, Option<PerpetualTriggerOrder>) {
    orders
        .iter()
        .filter(|o| o.is_position_tpsl && o.coin == coin)
        .fold((None, None), |(tp, sl), order| match order.trigger_px {
            Some(price) if order.order_type.to_lowercase().contains("take profit") => (
                Some(PerpetualTriggerOrder {
                    price,
                    order_type: determine_order_type(&order.order_type),
                    order_id: order.oid.to_string(),
                }),
                sl,
            ),
            Some(price) if order.order_type.to_lowercase().contains("stop") => (
                tp,
                Some(PerpetualTriggerOrder {
                    price,
                    order_type: determine_order_type(&order.order_type),
                    order_id: order.oid.to_string(),
                }),
            ),
            _ => (tp, sl),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        metadata::{AssetMetadata, HypercoreUniverseResponse, UniverseAsset},
        position::{AssetPosition, AssetPositions, CumulativeFunding, Leverage, LeverageType, MarginSummary, Position, PositionType},
    };
    use primitives::{PerpetualBalance, PerpetualDirection, PerpetualMarginType, PerpetualPosition, perpetual_provider::PerpetualProvider};

    #[test]
    fn test_map_positions_basic() {
        let positions = AssetPositions {
            asset_positions: vec![AssetPosition {
                position_type: PositionType::OneWay,
                position: Position {
                    coin: "BTC".to_string(),
                    szi: "1.5".to_string(),
                    leverage: Leverage {
                        leverage_type: LeverageType::Cross,
                        value: 10,
                    },
                    entry_px: "50000".to_string(),
                    position_value: "75000".to_string(),
                    unrealized_pnl: "5000".to_string(),
                    return_on_equity: "0.1".to_string(),
                    liquidation_px: Some("40000".to_string()),
                    margin_used: "7500".to_string(),
                    max_leverage: 20,
                    cum_funding: CumulativeFunding {
                        all_time: "100".to_string(),
                        since_open: "50".to_string(),
                    },
                },
            }],
            margin_summary: MarginSummary {
                account_value: "100000".to_string(),
                total_ntl_pos: "10000".to_string(),
                total_raw_usd: "10000".to_string(),
                total_margin_used: "5000".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "100000".to_string(),
                total_ntl_pos: "10000".to_string(),
                total_raw_usd: "10000".to_string(),
                total_margin_used: "8000".to_string(),
            },
            cross_maintenance_margin_used: "3000".to_string(),
            withdrawable: "92000".to_string(),
        };

        let result = map_positions(positions, "test_address", &[], 0);

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
            universe: vec![UniverseAsset {
                is_isolated_only: Some(false),
                ..UniverseAsset::mock()
            }],
        };

        let asset_metadata = vec![AssetMetadata {
            premium: Some("1.5".to_string()),
            impact_pxs: Some(vec!["2100".to_string(), "2105".to_string()]),
            ..AssetMetadata::mock()
        }];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let result = map_perpetuals_data(metadata_response, 0);

        assert_eq!(result.len(), 1);

        let eth_data = &result[0];
        assert_eq!(eth_data.perpetual.id, "hypercore_ETH");
        assert_eq!(eth_data.perpetual.name, "ETH");
        assert_eq!(eth_data.perpetual.provider, PerpetualProvider::Hypercore);
        assert_eq!(eth_data.perpetual.price, 2102.5);
        assert_eq!(eth_data.perpetual.funding, 0.05);
        assert_eq!(eth_data.perpetual.max_leverage, 50);
        assert_eq!(eth_data.perpetual.volume_24h, 500000.0);

        assert_eq!(eth_data.perpetual.is_isolated_only, false);

        assert_eq!(eth_data.asset.name, "ETH");
        assert_eq!(eth_data.asset.symbol, "ETH");
        assert_eq!(eth_data.asset.decimals, 4);
        assert_eq!(eth_data.asset.id.to_string(), "hypercore_perpetual::ETH");
    }

    #[test]
    fn test_map_perpetuals_data_builder_asset_index() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                name: "FOO".to_string(),
                ..UniverseAsset::mock()
            }],
        };

        let asset_metadata = vec![AssetMetadata::mock()];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let result = map_perpetuals_data(metadata_response, 2);

        assert_eq!(result[0].perpetual.identifier, "120000");
        assert_eq!(result[0].perpetual.id, "hypercore_2_FOO");
        assert_eq!(result[0].asset.id.to_string(), "hypercore_perpetual::2::FOO");
    }

    #[test]
    fn test_map_perpetuals_data_is_isolated_only() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![
                UniverseAsset {
                    name: "ISOLATED_TOKEN".to_string(),
                    is_isolated_only: Some(true),
                    ..UniverseAsset::mock()
                },
                UniverseAsset {
                    name: "DEFAULT_TOKEN".to_string(),
                    is_isolated_only: None,
                    ..UniverseAsset::mock()
                },
            ],
        };

        let asset_metadata = vec![AssetMetadata::mock(), AssetMetadata::mock()];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let result = map_perpetuals_data(metadata_response, 0);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].perpetual.is_isolated_only, true);
        assert_eq!(result[1].perpetual.is_isolated_only, false);
    }

    #[test]
    fn test_map_candlesticks() {
        use crate::models::candlestick::Candlestick;

        let candlesticks = vec![
            Candlestick {
                t: 1640995200000u64, // 2022-01-01 00:00:00 UTC
                s: "BTC".to_string(),
                i: "1h".to_string(),
                o: "50000.0".to_string(),
                h: "51000.0".to_string(),
                l: "49000.0".to_string(),
                c: "50500.0".to_string(),
                v: "100.5".to_string(),
            },
            Candlestick {
                t: 1640998800000u64, // 2022-01-01 01:00:00 UTC
                s: "BTC".to_string(),
                i: "1h".to_string(),
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
        let positions = AssetPositions {
            asset_positions: vec![
                AssetPosition {
                    position_type: PositionType::OneWay,
                    position: Position {
                        coin: "SOL".to_string(),
                        szi: "-10.0".to_string(),
                        leverage: Leverage {
                            leverage_type: LeverageType::Cross,
                            value: 20,
                        },
                        entry_px: "195.39".to_string(),
                        position_value: "2029.2".to_string(),
                        unrealized_pnl: "-75.3".to_string(),
                        return_on_equity: "-0.77076616".to_string(),
                        liquidation_px: Some("558.9517436098".to_string()),
                        margin_used: "101.46".to_string(),
                        max_leverage: 20,
                        cum_funding: CumulativeFunding {
                            all_time: "-1.3358".to_string(),
                            since_open: "-1.3".to_string(),
                        },
                    },
                },
                AssetPosition {
                    position_type: PositionType::OneWay,
                    position: Position {
                        coin: "BTC".to_string(),
                        szi: "3.0".to_string(),
                        leverage: Leverage {
                            leverage_type: LeverageType::Isolated,
                            value: 10,
                        },
                        entry_px: "766.34".to_string(),
                        position_value: "2332.2".to_string(),
                        unrealized_pnl: "33.18".to_string(),
                        return_on_equity: "0.1443223634".to_string(),
                        liquidation_px: None,
                        margin_used: "233.22".to_string(),
                        max_leverage: 10,
                        cum_funding: CumulativeFunding {
                            all_time: "1.686397".to_string(),
                            since_open: "1.1".to_string(),
                        },
                    },
                },
            ],
            margin_summary: MarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_maintenance_margin_used: "50".to_string(),
            withdrawable: "500".to_string(),
        };

        let summary = map_positions(positions, "test_user", &[], 0);

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
        let position = Position {
            szi: "3.0".to_string(),
            entry_px: "100".to_string(),
            position_value: "300".to_string(),
            margin_used: "30".to_string(),
            cum_funding: CumulativeFunding {
                all_time: "1.5".to_string(),
                since_open: "1.5".to_string(),
            },
            ..Position::mock()
        };

        let perpetual_position = map_position(position, "user123", &[], 0);
        assert_eq!(perpetual_position.funding, Some(-1.5)); // Long position reverses sign

        let short_position = Position {
            coin: "ETH".to_string(),
            szi: "-5.0".to_string(),
            entry_px: "100".to_string(),
            position_value: "500".to_string(),
            margin_used: "50".to_string(),
            cum_funding: CumulativeFunding {
                all_time: "-1.5".to_string(),
                since_open: "-1.5".to_string(),
            },
            ..Position::mock()
        };

        let short_perpetual = map_position(short_position, "user123", &[], 0);
        assert_eq!(short_perpetual.size, 5.0); // Size is always positive (absolute value)
        assert_eq!(short_perpetual.funding, Some(1.5)); // Short position with negative funding
    }

    #[test]
    fn test_map_perpetual_balance() {
        let positions = AssetPositions {
            asset_positions: vec![],
            margin_summary: MarginSummary {
                account_value: "5000.50".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "100".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "1000".to_string(),
                total_ntl_pos: "100".to_string(),
                total_raw_usd: "100".to_string(),
                total_margin_used: "1500.25".to_string(),
            },
            cross_maintenance_margin_used: "50".to_string(),
            withdrawable: "2500.75".to_string(),
        };

        let summary = map_positions(positions, "balance_test", &[], 0);

        assert_eq!(summary.balance.reserved, 1500.25);
        assert_eq!(summary.balance.available, 3500.25);
        assert_eq!(summary.balance.withdrawable, 2500.75);
    }

    #[test]
    fn test_map_perpetual_balance_with_real_data() {
        let positions = AssetPositions {
            asset_positions: vec![],
            margin_summary: MarginSummary {
                account_value: "706.364534".to_string(),
                total_ntl_pos: "12013.47849".to_string(),
                total_raw_usd: "2737.835324".to_string(),
                total_margin_used: "926.155026".to_string(),
            },
            cross_margin_summary: MarginSummary {
                account_value: "706.364534".to_string(),
                total_ntl_pos: "12013.47849".to_string(),
                total_raw_usd: "2737.835324".to_string(),
                total_margin_used: "926.155026".to_string(),
            },
            cross_maintenance_margin_used: "400.689965".to_string(),
            withdrawable: "305.674569".to_string(),
        };

        let summary = map_positions(positions, "real_data_test", &[], 0);

        assert_eq!(summary.balance.reserved, 706.364534);
        assert_eq!(summary.balance.available, 0.0);
        assert_eq!(summary.balance.withdrawable, 305.674569);
    }

    #[test]
    fn test_map_position_asset_id_uses_subtoken_pattern() {
        let position = Position::mock();

        let perpetual_position = map_position(position, "address123", &[], 0);

        assert_eq!(perpetual_position.asset_id.chain, primitives::Chain::HyperCore);
        assert_eq!(perpetual_position.asset_id.token_id, Some("perpetual::BTC".to_string()));
        assert_eq!(perpetual_position.asset_id.to_string(), "hypercore_perpetual::BTC");
    }

    #[test]
    fn test_map_position_builder_dex_ids() {
        let position = Position::mock();

        let perpetual_position = map_position(position, "address123", &[], 2);

        assert_eq!(perpetual_position.id, "address123_2_BTC");
        assert_eq!(perpetual_position.perpetual_id, "hypercore_2_BTC");
        assert_eq!(perpetual_position.asset_id.to_string(), "hypercore_perpetual::2::BTC");
    }

    #[test]
    fn test_map_position_uses_checksum_address_in_id() {
        let position = Position::mock();

        let perpetual_position = map_position(position, "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a", &[], 0);

        assert_eq!(perpetual_position.id, "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a_BTC");
    }

    #[test]
    fn test_map_tp_sl_from_orders_limit() {
        use crate::testkit::*;

        let orders = vec![
            OpenOrder::mock_with_trigger("HYPE", 191395165138, "Stop Limit", 35.0, Some(33.5)),
            OpenOrder::mock_with_trigger("HYPE", 191394991415, "Take Profit Limit", 55.0, Some(56.0)),
        ];

        let (take_profit, stop_loss) = map_tp_sl_from_orders(&orders, "HYPE");

        let tp = take_profit.unwrap();
        assert_eq!(tp.price, 55.0);
        assert_eq!(tp.order_type, PerpetualOrderType::Limit);
        assert_eq!(tp.order_id, "191394991415");

        let sl = stop_loss.unwrap();
        assert_eq!(sl.price, 35.0);
        assert_eq!(sl.order_type, PerpetualOrderType::Limit);
        assert_eq!(sl.order_id, "191395165138");
    }

    #[test]
    fn test_map_tp_sl_from_orders_market() {
        use crate::testkit::*;

        let orders = vec![
            OpenOrder::mock_with_trigger("BTC", 123456789, "Stop Market", 40000.0, None),
            OpenOrder::mock_with_trigger("BTC", 987654321, "Take Profit Market", 60000.0, None),
        ];

        let (take_profit, stop_loss) = map_tp_sl_from_orders(&orders, "BTC");

        let tp = take_profit.unwrap();
        assert_eq!(tp.price, 60000.0);
        assert_eq!(tp.order_type, PerpetualOrderType::Market);
        assert_eq!(tp.order_id, "987654321");

        let sl = stop_loss.unwrap();
        assert_eq!(sl.price, 40000.0);
        assert_eq!(sl.order_type, PerpetualOrderType::Market);
        assert_eq!(sl.order_id, "123456789");
    }

    #[test]
    fn test_map_perpetual_portfolio() {
        use crate::testkit::*;

        let response = HypercorePortfolioResponse {
            timeframes: vec![
                ("perpDay".to_string(), HypercorePortfolioTimeframeData::mock()),
                ("perpWeek".to_string(), HypercorePortfolioTimeframeData::mock_with_volume("500")),
                ("perpMonth".to_string(), HypercorePortfolioTimeframeData::mock_with_volume("2000")),
                ("perpAllTime".to_string(), HypercorePortfolioTimeframeData::mock_with_volume("50000")),
            ],
        };
        let positions = AssetPositions::mock();

        let result = map_perpetual_portfolio(response, &positions);

        assert_eq!(result.day.unwrap().volume, 100.0);
        assert_eq!(result.week.unwrap().volume, 500.0);
        assert_eq!(result.month.unwrap().volume, 2000.0);
        assert_eq!(result.all_time.unwrap().volume, 50000.0);

        let summary = result.account_summary.unwrap();
        assert_eq!(summary.account_value, 10000.0);
        assert_eq!(summary.account_leverage, 0.5);
        assert_eq!(summary.margin_usage, 0.2);
        assert_eq!(summary.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_map_account_summary() {
        use crate::testkit::*;

        let positions = AssetPositions::mock();
        let summary = map_account_summary(&positions);

        assert_eq!(summary.account_value, 10000.0);
        assert_eq!(summary.account_leverage, 0.5);
        assert_eq!(summary.margin_usage, 0.2);
        assert_eq!(summary.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_map_account_summary_aggregate() {
        use crate::testkit::*;

        let positions = vec![AssetPositions::mock(), AssetPositions::mock()];
        let summary = map_account_summary_aggregate(&positions);

        assert_eq!(summary.account_value, 20000.0);
        assert_eq!(summary.account_leverage, 0.5);
        assert_eq!(summary.margin_usage, 0.2);
        assert_eq!(summary.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_perp_asset_index() {
        assert_eq!(perp_asset_index(0, 0), 0);
        assert_eq!(perp_asset_index(0, 5), 5);
        assert_eq!(perp_asset_index(1, 0), 110_000);
        assert_eq!(perp_asset_index(1, 3), 110_003);
        assert_eq!(perp_asset_index(2, 0), 120_000);
        assert_eq!(perp_asset_index(2, 7), 120_007);
    }

    #[test]
    fn test_merge_chart_histories() {
        use chrono::{TimeZone, Utc};

        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let d3 = Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap();

        let histories = vec![
            vec![ChartDateValue { date: d1, value: 100.0 }, ChartDateValue { date: d2, value: 200.0 }],
            vec![ChartDateValue { date: d1, value: 50.0 }, ChartDateValue { date: d3, value: 300.0 }],
        ];

        let merged = merge_chart_histories(histories);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].value, 150.0);
        assert_eq!(merged[1].value, 200.0);
        assert_eq!(merged[2].value, 300.0);
    }

    #[test]
    fn test_merge_perpetual_portfolios() {
        let portfolios = vec![PerpetualPortfolio::mock(), PerpetualPortfolio::mock()];

        let summary = PerpetualAccountSummary {
            account_value: 1000.0,
            account_leverage: 2.0,
            margin_usage: 0.5,
            unrealized_pnl: 30.0,
        };

        let merged = merge_perpetual_portfolios(portfolios, Some(summary));

        let day = merged.day.unwrap();
        assert_eq!(day.volume, 10000.0);
        assert_eq!(day.account_value_history.len(), 1);
        assert_eq!(day.account_value_history[0].value, 2000.0);
        assert_eq!(day.pnl_history[0].value, 100.0);

        assert!(merged.week.is_none());

        let summary = merged.account_summary.unwrap();
        assert_eq!(summary.account_value, 1000.0);
    }

    #[test]
    fn test_merge_positions_summaries() {
        let second_position = PerpetualPosition::mock_with_values(
            "two",
            "hypercore_2_ETH",
            create_perpetual_asset_id_by_dex("ETH", 2),
            PerpetualDirection::Short,
            PerpetualMarginType::Isolated,
            2.0,
            200.0,
            4,
            200.0,
            30.0,
            5.0,
        );
        let summaries = vec![
            PerpetualPositionsSummary::mock(),
            PerpetualPositionsSummary::mock_with(vec![second_position], PerpetualBalance::mock_with_values(7.0, 3.0, 6.0)),
        ];

        let merged = merge_positions_summaries(summaries);

        assert_eq!(merged.positions.len(), 2);
        assert_eq!(merged.balance.available, 17.0);
        assert_eq!(merged.balance.reserved, 8.0);
        assert_eq!(merged.balance.withdrawable, 14.0);
    }
}
