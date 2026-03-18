use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};
use primitives::{Transaction, TransactionState};

use crate::models::action::{ACTION_ID_PREFIX, ExchangeRequest};
use crate::models::order::PerpetualFill;
use crate::models::response::{BroadcastResult, TransactionBroadcastResponse};
use crate::models::token::HYPERCORE_USDC_ASSET_ID;
use crate::perpetual_formatter::usdc_value;
use crate::provider::perpetual_mapper::create_perpetual_asset_id;
use crate::provider::transaction_state_mapper::prepare_perpetual_fill;

pub fn map_transaction_broadcast(response: serde_json::Value, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    let broadcast_response = serde_json::from_value::<TransactionBroadcastResponse>(response)?;
    let action_id = ExchangeRequest::get_nonce(&data).map(|nonce| format!("{}{}", ACTION_ID_PREFIX, nonce));
    match broadcast_response.into_result(action_id) {
        BroadcastResult::Success(result) => Ok(result),
        BroadcastResult::Error(error) => Err(error.into()),
    }
}

pub fn map_perpetual_fills(address: &str, fills: Vec<PerpetualFill>) -> Vec<Transaction> {
    let groups = fills.into_iter().fold(HashMap::<u64, Vec<PerpetualFill>>::new(), |mut acc, fill| {
        acc.entry(fill.oid).or_default().push(fill);
        acc
    });
    groups.into_values().filter_map(|fills| map_perpetual_fill_group(address, fills)).collect()
}

fn map_perpetual_fill_group(address: &str, fills: Vec<PerpetualFill>) -> Option<Transaction> {
    let last_fill = fills.iter().max_by_key(|fill| fill.time)?;
    let fill_refs = fills.iter().collect::<Vec<_>>();
    let (transaction_type, metadata) = prepare_perpetual_fill(&fill_refs, last_fill)?;
    let created_at = DateTime::<Utc>::from_timestamp_millis(last_fill.time as i64)?;
    let fee: f64 = fills.iter().map(|fill| fill.fee + fill.builder_fee.unwrap_or(0.0)).sum();
    let value: f64 = fills.iter().map(|fill| fill.px * fill.sz.parse::<f64>().unwrap_or(0.0)).sum();
    if last_fill.hash.is_empty() {
        return None;
    }
    let metadata = serde_json::to_value(metadata).ok()?;
    let address = address.to_string();

    Some(Transaction::new(
        last_fill.hash.clone(),
        create_perpetual_asset_id(&last_fill.coin),
        address.clone(),
        address,
        None,
        transaction_type,
        TransactionState::Confirmed,
        usdc_value(fee),
        HYPERCORE_USDC_ASSET_ID.clone(),
        usdc_value(value),
        None,
        Some(metadata),
        created_at,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::token::HYPERCORE_USDC_ASSET_ID;
    use primitives::{PerpetualDirection, TransactionPerpetualMetadata, TransactionType};

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/order_broadcast_filled.json")).unwrap();
        let data = include_str!("../../testdata/hl_action_open_long_order.json").trim().to_string();
        assert_eq!(map_transaction_broadcast(response, data).unwrap(), "134896397196");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/order_broadcast_error.json")).unwrap();
        let data = include_str!("../../testdata/hl_action_open_long_order.json").trim().to_string();
        assert!(map_transaction_broadcast(response, data).is_err());
    }

    #[test]
    fn test_map_transaction_broadcast_extra_agent_error() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error_extra_agent.json")).unwrap();
        let data = include_str!("../../testdata/hl_action_open_long_order.json").trim().to_string();
        let result = map_transaction_broadcast(response, data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Extra agent already used.");
    }

    #[test]
    fn test_map_transaction_broadcast_fallback_converts_json_to_action_nonce() {
        let response: serde_json::Value = serde_json::from_str(r#"{"status":"ok","response":{"type":"order"}}"#).unwrap();
        let data = include_str!("../../testdata/hl_action_update_position_tp_sl.json").trim().to_string();
        let result = map_transaction_broadcast(response, data).unwrap();
        assert_eq!(result, "action:1755132472149");
    }

    #[test]
    fn test_map_perpetual_fills_open_long_group() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let transactions = map_perpetual_fills("0xabc", fills);

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, TransactionType::PerpetualOpenPosition);
        assert_eq!(transactions[0].hash, "0x9b4d63110c57f2e19cc7042ce90e300202f500f6a75b11b33f160e63cb5bcccc");
        assert_eq!(transactions[0].asset_id.to_string(), "hypercore_perpetual::HYPE");
        assert_eq!(transactions[0].fee_asset_id, *HYPERCORE_USDC_ASSET_ID);
        assert_eq!(transactions[0].from, "0xabc");
        assert_eq!(transactions[0].to, "0xabc");

        let metadata: TransactionPerpetualMetadata = serde_json::from_value(transactions[0].metadata.clone().unwrap()).unwrap();
        assert_eq!(metadata.direction, PerpetualDirection::Long);
    }

    #[test]
    fn test_map_perpetual_fills_ignores_non_perpetual_direction() {
        let fills = vec![PerpetualFill {
            coin: "HYPE".to_string(),
            hash: "0xhash".to_string(),
            oid: 1,
            sz: "1".to_string(),
            closed_pnl: 0.0,
            fee: 0.1,
            builder_fee: None,
            px: 42.0,
            dir: "Buy".to_string(),
            time: 1,
        }];

        assert!(map_perpetual_fills("0xabc", fills).is_empty());
    }

    #[test]
    fn test_map_perpetual_fills_maps_liquidation_to_close() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_liquidation.json")).unwrap();
        let transactions = map_perpetual_fills("0xabc", fills);

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, TransactionType::PerpetualClosePosition);

        let metadata: TransactionPerpetualMetadata = serde_json::from_value(transactions[0].metadata.clone().unwrap()).unwrap();
        assert_eq!(metadata.direction, PerpetualDirection::Long);
    }

    #[test]
    fn test_map_perpetual_fills_keeps_distinct_oids_for_shared_hash() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_shared_hash.json")).unwrap();
        let transactions = map_perpetual_fills("0xabc", fills);

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].hash, "0xshared");
        assert_eq!(transactions[1].hash, "0xshared");
    }
}
