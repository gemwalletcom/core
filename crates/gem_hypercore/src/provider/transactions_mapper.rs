use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, Chain, SwapProvider, Transaction, TransactionState, TransactionSwapMetadata, TransactionType, asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID};

use crate::models::action::{ACTION_ID_PREFIX, ExchangeRequest};
use crate::models::order::{FillDirection, UserFill};
use crate::models::response::{BroadcastResult, TransactionBroadcastResponse};
use crate::models::spot::SpotMeta;
use crate::models::token::SpotToken;
use crate::perpetual_formatter::usdc_value;
use crate::provider::perpetual_mapper::create_perpetual_asset_id;
use crate::provider::transaction_state_mapper::prepare_perpetual_fill;

pub fn map_transaction_broadcast(response: serde_json::Value, data: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_value::<TransactionBroadcastResponse>(response)?;
    let action_id = ExchangeRequest::get_nonce(data).map(|nonce| format!("{ACTION_ID_PREFIX}{nonce}"));
    map_transaction_broadcast_result(response.into_result(action_id))
}

pub fn map_transaction_broadcast_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_str::<TransactionBroadcastResponse>(response)?;
    map_transaction_broadcast_result(response.into_result(None))
}

fn map_transaction_broadcast_result(result: BroadcastResult) -> Result<String, Box<dyn Error + Sync + Send>> {
    match result {
        BroadcastResult::Success(result) => Ok(result),
        BroadcastResult::Error(error) => Err(error.into()),
    }
}

pub fn map_user_fills(address: &str, fills: Vec<UserFill>, spot_meta: Option<&SpotMeta>) -> Vec<Transaction> {
    let groups = fills.into_iter().fold(HashMap::<u64, Vec<UserFill>>::new(), |mut acc, fill| {
        acc.entry(fill.oid).or_default().push(fill);
        acc
    });
    groups.into_values().filter_map(|fills| map_fill_group(address, fills, spot_meta)).collect()
}

fn map_fill_group(address: &str, fills: Vec<UserFill>, spot_meta: Option<&SpotMeta>) -> Option<Transaction> {
    let last_fill = fills.iter().max_by_key(|fill| fill.time)?.clone();

    match &last_fill.dir {
        FillDirection::Buy | FillDirection::Sell => map_spot_fill_group(address, fills, &last_fill, spot_meta),
        FillDirection::OpenLong | FillDirection::OpenShort | FillDirection::CloseLong | FillDirection::CloseShort | FillDirection::Other(_) => {
            map_perpetual_fill_group(address, fills, &last_fill)
        }
    }
}

fn map_perpetual_fill_group(address: &str, fills: Vec<UserFill>, last_fill: &UserFill) -> Option<Transaction> {
    let fill_refs = fills.iter().collect::<Vec<_>>();
    let (transaction_type, metadata) = prepare_perpetual_fill(&fill_refs, last_fill)?;
    let fee: f64 = fills.iter().map(|fill| fill.fee + fill.builder_fee.unwrap_or(0.0)).sum();
    let value = fills.iter().try_fold(0.0, |sum, fill| Some(sum + fill.px * fill.sz.parse::<f64>().ok()?))?;
    let metadata = serde_json::to_value(metadata).ok()?;

    build_fill_transaction(
        address,
        last_fill,
        create_perpetual_asset_id(&last_fill.coin),
        transaction_type,
        usdc_value(fee),
        HYPERCORE_SPOT_USDC_ASSET_ID.clone(),
        usdc_value(value),
        metadata,
    )
}

fn map_spot_fill_group(address: &str, fills: Vec<UserFill>, last_fill: &UserFill, spot_meta: Option<&SpotMeta>) -> Option<Transaction> {
    let spot_meta = spot_meta?;
    let market_index = last_fill.coin.strip_prefix('@')?.parse::<u32>().ok()?;
    let market = spot_meta.universe.iter().find(|market| market.index == market_index && market.tokens.len() == 2)?;
    let base_token = spot_meta.tokens.iter().find(|token| token.index == market.tokens[0])?;
    let quote_token = spot_meta.tokens.iter().find(|token| token.index == market.tokens[1])?;

    let (base_amount, quote_amount) = fills.iter().try_fold((0.0, 0.0), |(base_sum, quote_sum), fill| {
        let size = fill.sz.parse::<f64>().ok()?;
        Some((base_sum + size, quote_sum + fill.px * size))
    })?;
    let (fee, fee_asset_id) = map_spot_fee(&fills, base_token, quote_token)?;

    let ((from_token, from_amount), (to_token, to_amount)) = match &last_fill.dir {
        FillDirection::Sell => ((base_token, base_amount), (quote_token, quote_amount)),
        FillDirection::Buy => ((quote_token, quote_amount), (base_token, base_amount)),
        _ => return None,
    };
    let from_asset = from_token.asset_id(Chain::HyperCore);
    let from_value = amount_to_value(from_amount, from_token.wei_decimals)?;
    let to_asset = to_token.asset_id(Chain::HyperCore);
    let to_value = amount_to_value(to_amount, to_token.wei_decimals)?;

    let metadata = serde_json::to_value(TransactionSwapMetadata {
        from_asset: from_asset.clone(),
        from_value: from_value.clone(),
        to_asset,
        to_value,
        provider: Some(SwapProvider::Hyperliquid.id().to_string()),
    })
    .ok()?;

    build_fill_transaction(address, last_fill, from_asset, TransactionType::Swap, fee, fee_asset_id, from_value, metadata)
}

fn map_spot_fee(fills: &[UserFill], base_token: &SpotToken, quote_token: &SpotToken) -> Option<(String, primitives::AssetId)> {
    let fee_amount: f64 = fills.iter().map(|fill| fill.fee + fill.builder_fee.unwrap_or(0.0)).sum();
    let fee_token = fills.iter().rev().find_map(|fill| fill.fee_token.as_deref()).unwrap_or(quote_token.name.as_str());
    let fee_token = if fee_token == base_token.name { base_token } else { quote_token };

    Some((amount_to_value(fee_amount, fee_token.wei_decimals)?, fee_token.asset_id(Chain::HyperCore)))
}

fn amount_to_value(amount: f64, decimals: i32) -> Option<String> {
    let precision: usize = decimals.try_into().ok()?;
    BigNumberFormatter::value_from_amount(&format!("{amount:.precision$}"), precision as u32).ok()
}

fn build_fill_transaction(
    address: &str,
    last_fill: &UserFill,
    asset_id: AssetId,
    transaction_type: TransactionType,
    fee: String,
    fee_asset_id: AssetId,
    value: String,
    metadata: serde_json::Value,
) -> Option<Transaction> {
    if last_fill.hash.is_empty() {
        return None;
    }

    let created_at = DateTime::<Utc>::from_timestamp_millis(last_fill.time as i64)?;
    let address = address.to_string();

    Some(Transaction::new(
        last_fill.hash.clone(),
        asset_id,
        address.clone(),
        address,
        None,
        transaction_type,
        TransactionState::Confirmed,
        fee,
        fee_asset_id,
        value,
        None,
        Some(metadata),
        created_at,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::spot::SpotMeta;
    use primitives::{PerpetualDirection, TransactionPerpetualMetadata, TransactionType, asset_constants::HYPERCORE_SPOT_HYPE_ASSET_ID};

    fn spot_meta() -> SpotMeta {
        serde_json::from_str(include_str!("../../testdata/spot_meta_spot_swap.json")).unwrap()
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/order_broadcast_filled.json")).unwrap();
        let data = include_str!("../../testdata/hl_action_open_long_order.json").trim().to_string();
        assert_eq!(map_transaction_broadcast(response, &data).unwrap(), "134896397196");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/order_broadcast_error.json")).unwrap();
        let data = include_str!("../../testdata/hl_action_open_long_order.json").trim().to_string();
        assert!(map_transaction_broadcast(response, &data).is_err());
    }

    #[test]
    fn test_map_transaction_broadcast_extra_agent_error() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error_extra_agent.json")).unwrap();
        let data = include_str!("../../testdata/hl_action_open_long_order.json").trim().to_string();
        let result = map_transaction_broadcast(response, &data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Extra agent already used.");
    }

    #[test]
    fn test_map_transaction_broadcast_fallback_converts_json_to_action_nonce() {
        let response: serde_json::Value = serde_json::from_str(r#"{"status":"ok","response":{"type":"order"}}"#).unwrap();
        let data = include_str!("../../testdata/hl_action_update_position_tp_sl.json").trim().to_string();
        let result = map_transaction_broadcast(response, &data).unwrap();
        assert_eq!(result, "action:1755132472149");
    }

    #[test]
    fn test_map_perpetual_fills_open_long_group() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let transactions = map_user_fills("0xabc", fills, None);

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, TransactionType::PerpetualOpenPosition);
        assert_eq!(transactions[0].hash, "0x9b4d63110c57f2e19cc7042ce90e300202f500f6a75b11b33f160e63cb5bcccc");
        assert_eq!(transactions[0].asset_id.to_string(), "hypercore_perpetual::HYPE");
        assert_eq!(transactions[0].fee_asset_id, HYPERCORE_SPOT_USDC_ASSET_ID.clone());
        assert_eq!(transactions[0].from, "0xabc");
        assert_eq!(transactions[0].to, "0xabc");

        let metadata: TransactionPerpetualMetadata = serde_json::from_value(transactions[0].metadata.clone().unwrap()).unwrap();
        assert_eq!(metadata.direction, PerpetualDirection::Long);
        assert_eq!(metadata.is_liquidation, Some(false));
    }

    #[test]
    fn test_map_perpetual_fills_ignores_unknown_direction() {
        let fills = vec![UserFill {
            coin: "HYPE".to_string(),
            hash: "0xhash".to_string(),
            oid: 1,
            sz: "1".to_string(),
            closed_pnl: 0.0,
            fee: 0.1,
            builder_fee: None,
            fee_token: None,
            px: 42.0,
            dir: FillDirection::Other("Unknown".to_string()),
            time: 1,
            liquidation: None,
        }];

        assert!(map_user_fills("0xabc", fills, Some(&spot_meta())).is_empty());
    }

    #[test]
    fn test_map_perpetual_fills_maps_liquidation_to_close() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_liquidation.json")).unwrap();
        let transactions = map_user_fills("0xabc", fills, Some(&spot_meta()));

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, TransactionType::PerpetualClosePosition);

        let metadata: TransactionPerpetualMetadata = serde_json::from_value(transactions[0].metadata.clone().unwrap()).unwrap();
        assert_eq!(metadata.direction, PerpetualDirection::Long);
        assert_eq!(metadata.is_liquidation, Some(true));
    }

    #[test]
    fn test_map_perpetual_fills_keeps_distinct_oids_for_shared_hash() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_shared_hash.json")).unwrap();
        let transactions = map_user_fills("0xabc", fills, Some(&spot_meta()));

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].hash, "0xshared");
        assert_eq!(transactions[1].hash, "0xshared");
    }

    #[test]
    fn test_map_spot_swap_fill_group() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_spot_swap.json")).unwrap();
        let transactions = map_user_fills("0xabc", fills, Some(&spot_meta()));

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, TransactionType::Swap);
        assert_eq!(transactions[0].hash, "0xd16518b18533f577d2de043763f8ad020482009720371449752dc4044437cf62");
        assert_eq!(transactions[0].asset_id, HYPERCORE_SPOT_HYPE_ASSET_ID.clone());
        assert_eq!(transactions[0].fee, "1858810");
        assert_eq!(transactions[0].fee_asset_id, HYPERCORE_SPOT_USDC_ASSET_ID.clone());
        assert!(transactions[0].asset_ids().contains(&HYPERCORE_SPOT_USDC_ASSET_ID.clone()));
        assert!(transactions[0].asset_ids().contains(&HYPERCORE_SPOT_HYPE_ASSET_ID.clone()));

        let metadata: TransactionSwapMetadata = serde_json::from_value(transactions[0].metadata.clone().unwrap()).unwrap();
        assert_eq!(metadata.from_asset, HYPERCORE_SPOT_HYPE_ASSET_ID.clone());
        assert_eq!(metadata.from_value, "30000000");
        assert_eq!(metadata.to_asset, HYPERCORE_SPOT_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, "1182450000");
        assert_eq!(metadata.provider.as_deref(), Some("hyperliquid"));
    }

    #[test]
    fn test_map_spot_buy_fill_group() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_spot_swap_buy.json")).unwrap();
        let transactions = map_user_fills("0xabc", fills, Some(&spot_meta()));

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, TransactionType::Swap);
        assert_eq!(transactions[0].hash, "0xbf8b52bd13095a59c105043764964e02028200a2ae0c792b6353fe0fd20d3444");
        assert_eq!(transactions[0].asset_id, HYPERCORE_SPOT_USDC_ASSET_ID.clone());
        assert_eq!(transactions[0].fee, "20159");
        assert_eq!(transactions[0].fee_asset_id, HYPERCORE_SPOT_HYPE_ASSET_ID.clone());
        assert!(transactions[0].asset_ids().contains(&HYPERCORE_SPOT_USDC_ASSET_ID.clone()));
        assert!(transactions[0].asset_ids().contains(&HYPERCORE_SPOT_HYPE_ASSET_ID.clone()));

        let metadata: TransactionSwapMetadata = serde_json::from_value(transactions[0].metadata.clone().unwrap()).unwrap();
        assert_eq!(metadata.from_asset, HYPERCORE_SPOT_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, "1197900000");
        assert_eq!(metadata.to_asset, HYPERCORE_SPOT_HYPE_ASSET_ID.clone());
        assert_eq!(metadata.to_value, "30000000");
        assert_eq!(metadata.provider.as_deref(), Some("hyperliquid"));
    }
}
