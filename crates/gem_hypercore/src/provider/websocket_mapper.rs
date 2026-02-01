use std::collections::{HashMap, HashSet};

use primitives::{AssetId, PerpetualPosition};

use crate::models::{
    order::OpenOrder,
    websocket::{HyperliquidSocketMessage, PositionsDiff, RawSocketMessage},
};

use super::perpetual_mapper::{map_positions, map_tp_sl_from_orders};

pub fn parse_websocket_data(data: &[u8]) -> Result<HyperliquidSocketMessage, serde_json::Error> {
    let raw: RawSocketMessage = serde_json::from_slice(data)?;

    match raw {
        RawSocketMessage::ClearinghouseState(data) => {
            let summary = map_positions(data.clearinghouse_state, data.user, &[]);
            Ok(HyperliquidSocketMessage::ClearinghouseState {
                balance: summary.balance,
                positions: summary.positions,
            })
        }
        RawSocketMessage::OpenOrders(data) => Ok(HyperliquidSocketMessage::OpenOrders { orders: data.orders }),
        RawSocketMessage::Candle(candlestick) => Ok(HyperliquidSocketMessage::Candle { candle: candlestick.into() }),
        RawSocketMessage::AllMids(data) => {
            let prices = data.mids.into_iter().filter_map(|(k, v)| v.parse::<f64>().ok().map(|p| (k, p))).collect();
            Ok(HyperliquidSocketMessage::AllMids { prices })
        }
        RawSocketMessage::SubscriptionResponse(data) => Ok(HyperliquidSocketMessage::SubscriptionResponse {
            subscription_type: data.subscription.subscription_type,
        }),
        RawSocketMessage::Unknown => Ok(HyperliquidSocketMessage::Unknown),
    }
}

pub fn diff_clearinghouse_positions(new_positions: Vec<PerpetualPosition>, existing_positions: Vec<PerpetualPosition>) -> PositionsDiff {
    let existing_map: HashMap<&str, &PerpetualPosition> = existing_positions.iter().map(|p| (p.id.as_str(), p)).collect();

    let positions: Vec<PerpetualPosition> = new_positions
        .into_iter()
        .map(|pos| match existing_map.get(pos.id.as_str()) {
            Some(existing) => PerpetualPosition {
                take_profit: existing.take_profit.clone(),
                stop_loss: existing.stop_loss.clone(),
                ..pos
            },
            None => pos,
        })
        .collect();

    let new_ids: HashSet<&str> = positions.iter().map(|p| p.id.as_str()).collect();
    let delete_position_ids: Vec<String> = existing_positions.iter().filter(|p| !new_ids.contains(p.id.as_str())).map(|p| p.id.clone()).collect();

    PositionsDiff { delete_position_ids, positions }
}

pub fn diff_open_orders_positions(orders: &[OpenOrder], existing_positions: Vec<PerpetualPosition>) -> PositionsDiff {
    let positions: Vec<PerpetualPosition> = existing_positions
        .into_iter()
        .filter_map(|pos| {
            let coin = pos.asset_id.token_id.as_ref().and_then(|t| AssetId::decode_token_id(t).into_iter().nth(1))?;
            let (take_profit, stop_loss) = map_tp_sl_from_orders(orders, &coin);

            if pos.take_profit != take_profit || pos.stop_loss != stop_loss {
                Some(PerpetualPosition { take_profit, stop_loss, ..pos })
            } else {
                None
            }
        })
        .collect();

    PositionsDiff {
        delete_position_ids: vec![],
        positions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, PerpetualDirection, PerpetualMarginType};

    #[test]
    fn test_parse_all_mids() {
        let json = include_bytes!("../../testdata/ws_all_mids.json");
        let HyperliquidSocketMessage::AllMids { prices } = parse_websocket_data(json).unwrap() else {
            panic!("expected AllMids");
        };

        assert_eq!(prices.len(), 5);
        assert_eq!(prices["BTC"], 104633.0);
        assert_eq!(prices["ETH"], 3321.1);
        assert_eq!(prices["SOL"], 260.48);
        assert_eq!(prices["DOGE"], 0.40381);
        assert_eq!(prices["HYPE"], 26.65);
    }

    #[test]
    fn test_parse_candle() {
        let json = include_bytes!("../../testdata/ws_candle.json");
        let HyperliquidSocketMessage::Candle { candle } = parse_websocket_data(json).unwrap() else {
            panic!("expected Candle");
        };

        assert_eq!(candle.interval, "1h");
        assert_eq!(candle.open, 3300.5);
        assert_eq!(candle.close, 3321.1);
        assert_eq!(candle.high, 3345.0);
        assert_eq!(candle.low, 3290.2);
        assert_eq!(candle.volume, 12450.8);
    }

    #[test]
    fn test_parse_open_orders() {
        let json = include_bytes!("../../testdata/ws_open_orders.json");
        let HyperliquidSocketMessage::OpenOrders { orders } = parse_websocket_data(json).unwrap() else {
            panic!("expected OpenOrders");
        };

        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].coin, "BTC");
        assert_eq!(orders[0].oid, 8804521338);
        assert_eq!(orders[0].trigger_px, Some(110000.0));
        assert_eq!(orders[0].limit_px, Some(110000.0));
        assert_eq!(orders[0].is_position_tpsl, true);
        assert_eq!(orders[0].order_type, "Take Profit Market");
        assert_eq!(orders[1].coin, "BTC");
        assert_eq!(orders[1].oid, 8804521339);
        assert_eq!(orders[1].trigger_px, Some(95000.0));
        assert_eq!(orders[1].limit_px, Some(95000.0));
        assert_eq!(orders[1].is_position_tpsl, true);
        assert_eq!(orders[1].order_type, "Stop Market");
    }

    #[test]
    fn test_parse_clearinghouse_state() {
        let json = include_bytes!("../../testdata/ws_clearinghouse_state.json");
        let HyperliquidSocketMessage::ClearinghouseState { balance, positions } = parse_websocket_data(json).unwrap() else {
            panic!("expected ClearinghouseState");
        };

        assert_eq!(balance.available, 15230.5 - 830.5);
        assert_eq!(balance.reserved, 830.5);
        assert_eq!(balance.withdrawable, 14400.0);

        assert_eq!(positions.len(), 1);
        let pos = &positions[0];
        assert_eq!(pos.id, "0xc64cc00b46150e2681a6c0e57b4b12fd2b68fbc4_ETH");
        assert_eq!(pos.asset_id.chain, Chain::HyperCore);
        assert_eq!(pos.size, 2.5);
        assert_eq!(pos.size_value, 8305.0);
        assert_eq!(pos.leverage, 10);
        assert_eq!(pos.entry_price, 3200.0);
        assert_eq!(pos.liquidation_price, Some(2850.5));
        assert_eq!(pos.margin_type, PerpetualMarginType::Cross);
        assert_eq!(pos.direction, PerpetualDirection::Long);
        assert_eq!(pos.margin_amount, 830.5);
        assert_eq!(pos.pnl, 305.0);
        assert_eq!(pos.funding, Some(-1.82));
        assert_eq!(pos.take_profit, None);
        assert_eq!(pos.stop_loss, None);
    }
}
