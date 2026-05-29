use number_formatter::BigNumberFormatter;
use primitives::{
    PerpetualDirection, PerpetualProvider, TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionType, TransactionUpdate,
    known_assets::HYPERCORE_HYPE,
};

use crate::models::{
    order::{FillDirection, UserFill},
    transaction_id::HyperCoreActionId,
    user::{DelegatorHistoryDelta, DelegatorHistoryUpdate, LedgerDelta, LedgerUpdate},
};
use crate::perpetual_formatter::usdc_value;

pub const ACTION_HISTORY_QUERY_LOOKBACK_MS: u64 = 5_000;
const ACTION_HISTORY_MATCH_WINDOW_MS: u64 = 5 * 60 * 1_000;
const DELEGATOR_WITHDRAWAL_INITIATED: &str = "initiated";

fn perpetual_fill_type_and_direction(dir: &FillDirection) -> Option<(TransactionType, PerpetualDirection)> {
    match dir {
        FillDirection::OpenLong => Some((TransactionType::PerpetualOpenPosition, PerpetualDirection::Long)),
        FillDirection::OpenShort => Some((TransactionType::PerpetualOpenPosition, PerpetualDirection::Short)),
        FillDirection::CloseLong => Some((TransactionType::PerpetualClosePosition, PerpetualDirection::Long)),
        FillDirection::CloseShort => Some((TransactionType::PerpetualClosePosition, PerpetualDirection::Short)),
        FillDirection::Buy | FillDirection::Sell | FillDirection::Other(_) => None,
    }
}

pub fn prepare_perpetual_fill(matching_fills: &[&UserFill], last_fill: &UserFill) -> Option<(TransactionType, TransactionPerpetualMetadata)> {
    let (transaction_type, direction) = perpetual_fill_type_and_direction(&last_fill.dir)?;
    let pnl: f64 = matching_fills.iter().map(|fill| fill.closed_pnl).sum();
    let is_liquidation = matching_fills.iter().any(|fill| fill.liquidation.is_some());

    Some((
        transaction_type,
        TransactionPerpetualMetadata {
            pnl,
            price: last_fill.px,
            direction,
            is_liquidation: Some(is_liquidation),
            provider: Some(PerpetualProvider::Hypercore),
        },
    ))
}

pub fn map_transaction_state_order(fills: Vec<UserFill>, oid: u64, request_id: String) -> TransactionUpdate {
    let matching_fills: Vec<_> = fills.iter().filter(|fill| fill.oid == oid).collect();

    let last_fill = match matching_fills.last() {
        Some(fill) => fill,
        None => return TransactionUpdate::new_state(TransactionState::Pending),
    };

    let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);

    match &last_fill.dir {
        FillDirection::Buy | FillDirection::Sell => {}
        FillDirection::OpenLong | FillDirection::OpenShort | FillDirection::CloseLong | FillDirection::CloseShort => {
            if let Some(changes) = perpetual_fill_changes(&matching_fills, last_fill) {
                update.changes.extend(changes);
            }
        }
        FillDirection::Other(_) => return TransactionUpdate::new_state(TransactionState::Pending),
    }

    if !last_fill.hash.is_empty() && last_fill.hash != request_id {
        update.changes.push(TransactionChange::HashChange {
            old: request_id,
            new: last_fill.hash.clone(),
        });
    }

    update
}

pub fn map_transaction_state_order_action(fills: Vec<UserFill>, nonce: u64, request_id: String) -> TransactionUpdate {
    let Some(fill) = order_action_fill(&fills, nonce) else {
        return TransactionUpdate::new_state(TransactionState::Pending);
    };

    let hash = fill.hash.clone();
    let matching_fills: Vec<_> = fills.iter().filter(|item| item.oid == fill.oid).collect();
    let mut update = TransactionUpdate::new(TransactionState::Confirmed, vec![TransactionChange::HashChange { old: request_id, new: hash }]);

    if let Some(last_fill) = matching_fills.iter().max_by_key(|fill| fill.time)
        && let Some(changes) = perpetual_fill_changes(&matching_fills, last_fill)
    {
        update.changes.extend(changes);
    }

    update
}

pub(crate) fn order_action_fill(fills: &[UserFill], nonce: u64) -> Option<&UserFill> {
    fills
        .iter()
        .filter_map(|fill| action_history_time_delta(fill.time, nonce).filter(|_| !fill.hash.is_empty()).map(|delta| (delta, fill)))
        .min_by_key(|(delta, _)| *delta)
        .map(|(_, fill)| fill)
}

pub fn map_transaction_state_action(updates: Vec<LedgerUpdate>, action_id: HyperCoreActionId, request_id: String) -> TransactionUpdate {
    transaction_update_from_hash(ledger_action_hash(&updates, &action_id), request_id)
}

pub fn map_transaction_state_staking_action(updates: Vec<DelegatorHistoryUpdate>, action_id: HyperCoreActionId, request_id: String) -> TransactionUpdate {
    transaction_update_from_hash(delegator_history_action_hash(&updates, &action_id), request_id)
}

pub fn ledger_action_hash(updates: &[LedgerUpdate], action_id: &HyperCoreActionId) -> Option<String> {
    let nonce = action_id.nonce();
    updates
        .iter()
        .filter_map(|update| ledger_match_delta(update, action_id, nonce).map(|delta| (delta, update)))
        .min_by_key(|(delta, _)| *delta)
        .map(|(_, update)| update.hash.clone())
}

fn delegator_history_action_hash(updates: &[DelegatorHistoryUpdate], action_id: &HyperCoreActionId) -> Option<String> {
    let nonce = action_id.nonce();
    updates
        .iter()
        .filter_map(|update| delegator_history_match_delta(update, action_id, nonce).map(|delta| (delta, update)))
        .min_by_key(|(delta, _)| *delta)
        .map(|(_, update)| update.hash.clone())
}

fn ledger_match_delta(update: &LedgerUpdate, action_id: &HyperCoreActionId, nonce: u64) -> Option<u64> {
    match &update.delta {
        LedgerDelta::Send { nonce: update_nonce } | LedgerDelta::SpotTransfer { nonce: update_nonce } if *update_nonce == nonce => Some(0),
        LedgerDelta::CStakingTransfer { token, amount, is_deposit } => {
            let (wei, expected_deposit) = match action_id {
                HyperCoreActionId::CDeposit { wei, .. } => (*wei, true),
                HyperCoreActionId::CWithdraw { wei, .. } => (*wei, false),
                HyperCoreActionId::TokenDelegate { wei, is_undelegate, .. } => (*wei, !*is_undelegate),
                HyperCoreActionId::Nonce(_) | HyperCoreActionId::Order(_) => return None,
            };

            if token != HYPERCORE_HYPE.symbol.as_str() || *is_deposit != expected_deposit {
                return None;
            }

            action_history_time_delta(update.time, nonce).filter(|_| amount_matches_wei(amount, wei))
        }
        LedgerDelta::Send { .. } | LedgerDelta::SpotTransfer { .. } | LedgerDelta::Other => None,
    }
}

fn delegator_history_match_delta(update: &DelegatorHistoryUpdate, action_id: &HyperCoreActionId, nonce: u64) -> Option<u64> {
    let matches_action = match (&update.delta, action_id) {
        (DelegatorHistoryDelta { c_deposit: Some(delta), .. }, HyperCoreActionId::CDeposit { wei, .. }) => amount_matches_wei(&delta.amount, *wei),
        (DelegatorHistoryDelta { delegate: Some(delta), .. }, HyperCoreActionId::TokenDelegate { wei, is_undelegate, .. }) => {
            delta.is_undelegate == *is_undelegate && amount_matches_wei(&delta.amount, *wei)
        }
        (DelegatorHistoryDelta { withdrawal: Some(delta), .. }, HyperCoreActionId::CWithdraw { wei, .. }) => {
            delta.phase == DELEGATOR_WITHDRAWAL_INITIATED && amount_matches_wei(&delta.amount, *wei)
        }
        _ => false,
    };

    if matches_action { action_history_time_delta(update.time, nonce) } else { None }
}

fn amount_matches_wei(amount: &str, wei: u64) -> bool {
    match BigNumberFormatter::value_from_amount(amount, HYPERCORE_HYPE.decimals as u32) {
        Ok(update_wei) => update_wei == wei.to_string(),
        Err(_) => false,
    }
}

fn action_history_time_delta(time: u64, nonce: u64) -> Option<u64> {
    let delta = time.checked_sub(nonce)?;
    if delta <= ACTION_HISTORY_MATCH_WINDOW_MS { Some(delta) } else { None }
}

fn transaction_update_from_hash(hash: Option<String>, request_id: String) -> TransactionUpdate {
    match hash {
        Some(hash) => TransactionUpdate::new(TransactionState::Confirmed, vec![TransactionChange::HashChange { old: request_id, new: hash }]),
        None => TransactionUpdate::new_state(TransactionState::Pending),
    }
}

fn perpetual_fill_changes(matching_fills: &[&UserFill], last_fill: &UserFill) -> Option<Vec<TransactionChange>> {
    let (_, metadata) = prepare_perpetual_fill(matching_fills, last_fill)?;
    let fee: f64 = matching_fills.iter().map(|fill| fill.fee + fill.builder_fee.unwrap_or(0.0)).sum();
    let network_fee = usdc_value(fee).parse().ok()?;

    Some(vec![
        TransactionChange::Metadata(TransactionMetadata::Perpetual(metadata)),
        TransactionChange::NetworkFee(network_fee),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::order::{FillDirection, UserFill};
    use num_bigint::BigInt;

    #[test]
    fn test_map_transaction_state_order() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let oid = 187530505765u64;
        let request_id = oid.to_string();

        let update = map_transaction_state_order(fills, oid, request_id.clone());

        assert_eq!(update.state, TransactionState::Confirmed);
        assert_eq!(update.changes.len(), 3);

        let metadata_change = update.changes.iter().find_map(|change| {
            if let TransactionChange::Metadata(TransactionMetadata::Perpetual(metadata)) = change {
                Some(metadata)
            } else {
                None
            }
        });
        let metadata = metadata_change.unwrap();
        assert_eq!(metadata.pnl, 36.5);
        assert_eq!(metadata.price, 47.904);
        assert_eq!(metadata.direction, PerpetualDirection::Long);
        assert_eq!(metadata.is_liquidation, Some(false));
        assert_eq!(metadata.provider, Some(PerpetualProvider::Hypercore));

        let network_fee_change = update
            .changes
            .iter()
            .find_map(|change| if let TransactionChange::NetworkFee(fee) = change { Some(fee) } else { None });
        assert_eq!(network_fee_change, Some(&BigInt::from(666786)));

        let hash_change = update.changes.iter().find_map(|change| {
            if let TransactionChange::HashChange { old, new } = change {
                Some((old, new))
            } else {
                None
            }
        });
        let (old, new) = hash_change.unwrap();
        assert_eq!(old, &request_id);
        assert_eq!(new, "0x9b4d63110c57f2e19cc7042ce90e300202f500f6a75b11b33f160e63cb5bcccc");
    }

    #[test]
    fn test_map_transaction_state_order_no_matching_fills() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let update = map_transaction_state_order(fills, 999999999u64, "999999999".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_state_order_non_perpetual_fill_stays_pending() {
        let fills = vec![UserFill {
            coin: "HYPE".to_string(),
            hash: String::new(),
            oid: 123,
            sz: "1".to_string(),
            closed_pnl: 0.0,
            fee: 0.0,
            builder_fee: None,
            fee_token: None,
            px: 42.0,
            dir: FillDirection::Other(String::new()),
            time: 0,
            liquidation: None,
        }];

        let update = map_transaction_state_order(fills, 123, "123".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_state_order_spot_fill_confirms() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_spot_swap.json")).unwrap();

        let request_id = "355101232455".to_string();
        let update = map_transaction_state_order(fills, 355101232455, request_id.clone());

        assert_eq!(update.state, TransactionState::Confirmed);
        assert_eq!(update.changes.len(), 1);
        assert_eq!(
            update.changes[0],
            TransactionChange::HashChange {
                old: request_id,
                new: "0xd16518b18533f577d2de043763f8ad020482009720371449752dc4044437cf62".to_string(),
            }
        );
    }

    #[test]
    fn test_map_transaction_state_order_action_includes_perpetual_fee() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let request_id = "action:order:1759700579000".to_string();
        let update = map_transaction_state_order_action(fills, 1759700579000, request_id.clone());

        assert_eq!(update.state, TransactionState::Confirmed);
        assert_eq!(
            update.changes[0],
            TransactionChange::HashChange {
                old: request_id,
                new: "0x9b4d63110c57f2e19cc7042ce90e300202f500f6a75b11b33f160e63cb5bcccc".to_string(),
            }
        );
        let network_fee_change = update
            .changes
            .iter()
            .find_map(|change| if let TransactionChange::NetworkFee(fee) = change { Some(fee) } else { None });
        assert_eq!(network_fee_change, Some(&BigInt::from(666786)));
    }

    #[test]
    fn test_map_transaction_state_order_action_confirms_closest_fill() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_spot_swap.json")).unwrap();
        let request_id = "action:order:1773977221000".to_string();
        let update = map_transaction_state_order_action(fills, 1773977221000, request_id.clone());

        assert_eq!(
            update,
            TransactionUpdate::new(
                TransactionState::Confirmed,
                vec![TransactionChange::HashChange {
                    old: request_id,
                    new: "0xd16518b18533f577d2de043763f8ad020482009720371449752dc4044437cf62".to_string(),
                }]
            )
        );
    }

    #[test]
    fn test_map_transaction_state_action_without_matching_nonce_stays_pending() {
        let updates = serde_json::from_str(include_str!("../../testdata/user_non_funding_ledger_updates_action_hash.json")).unwrap();
        let update = map_transaction_state_action(updates, HyperCoreActionId::Nonce(1777960893093), "action:1777960893093".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_state_action_confirms_with_hash_change() {
        let updates = serde_json::from_str(include_str!("../../testdata/user_non_funding_ledger_updates_action_hash.json")).unwrap();
        let request_id = "action:1777960893092".to_string();
        let update = map_transaction_state_action(updates, HyperCoreActionId::Nonce(1777960893092), request_id.clone());

        assert_eq!(
            update,
            TransactionUpdate::new(
                TransactionState::Confirmed,
                vec![TransactionChange::HashChange {
                    old: request_id,
                    new: "0xba3bce0950157157bbb5043aaee1060201e300eeeb1890295e04795c0f194b42".to_string(),
                }]
            )
        );
    }

    #[test]
    fn test_map_transaction_state_action_confirms_spot_transfer_nonce() {
        let updates = serde_json::from_str(include_str!("../../testdata/user_non_funding_ledger_updates_spot_transfer.json")).unwrap();
        let request_id = "action:1761611679622".to_string();
        let update = map_transaction_state_action(updates, HyperCoreActionId::Nonce(1761611679622), request_id.clone());

        assert_eq!(
            update,
            TransactionUpdate::new(
                TransactionState::Confirmed,
                vec![TransactionChange::HashChange {
                    old: request_id,
                    new: "0x1210f05525bce189138a042e558d8002126b003ac0b0005bb5d99ba7e4b0bb73".to_string(),
                }]
            )
        );
    }

    #[test]
    fn test_map_transaction_state_action_confirms_staking_transfer_with_typed_action() {
        let updates = serde_json::from_str(include_str!("../../testdata/user_non_funding_ledger_updates_c_staking_transfer.json")).unwrap();
        let request_id = "action:cDeposit:1000000:1779376553779".to_string();
        let action_id = HyperCoreActionId::CDeposit {
            wei: 1_000_000,
            nonce: 1779376553779,
        };
        let update = map_transaction_state_action(updates, action_id, request_id.clone());

        assert_eq!(
            update,
            TransactionUpdate::new(
                TransactionState::Confirmed,
                vec![TransactionChange::HashChange {
                    old: request_id,
                    new: "0xf0515f4aee4cd625f1cb043be9536a0203ca0030894ff4f8941a0a9dad40b010".to_string(),
                }]
            )
        );
    }

    #[test]
    fn test_map_transaction_state_action_resolves_token_delegate_to_staking_transfer() {
        let updates = serde_json::from_str(include_str!("../../testdata/user_non_funding_ledger_updates_c_staking_transfer.json")).unwrap();
        let request_id = "action:tokenDelegate:1000000:stake:1779376553780".to_string();
        let action_id = HyperCoreActionId::TokenDelegate {
            wei: 1_000_000,
            is_undelegate: false,
            nonce: 1779376553780,
        };
        let update = map_transaction_state_action(updates, action_id, request_id.clone());

        assert_eq!(
            update,
            TransactionUpdate::new(
                TransactionState::Confirmed,
                vec![TransactionChange::HashChange {
                    old: request_id,
                    new: "0xf0515f4aee4cd625f1cb043be9536a0203ca0030894ff4f8941a0a9dad40b010".to_string(),
                }]
            )
        );
    }

    #[test]
    fn test_map_transaction_state_staking_action_confirms_delegator_history_actions() {
        let updates: Vec<DelegatorHistoryUpdate> = serde_json::from_str(include_str!("../../testdata/delegator_history_staking_actions.json")).unwrap();

        for (request_id, action_id, expected_hash) in [
            (
                "action:cDeposit:1000000:1780081714468",
                HyperCoreActionId::CDeposit {
                    wei: 1_000_000,
                    nonce: 1780081714468,
                },
                "0x945b910697cd885a95d5043c857c0d0201b300ec32c0a72c38243c5956c16245",
            ),
            (
                "action:tokenDelegate:1000000:stake:1780081715280",
                HyperCoreActionId::TokenDelegate {
                    wei: 1_000_000,
                    is_undelegate: false,
                    nonce: 1780081715280,
                },
                "0x0cfde0fb239ef8630e77043c857c1502025000e0be921735b0c68c4de292d24d",
            ),
            (
                "action:cWithdraw:3001423:1780078264489",
                HyperCoreActionId::CWithdraw {
                    wei: 3_001_423,
                    nonce: 1780078264489,
                },
                "0x7b435a1210afafef7cbd043c84b8d402064e00f7aba2cec11f0c0564cfa389da",
            ),
            (
                "action:tokenDelegate:3001423:unstake:1780078264488",
                HyperCoreActionId::TokenDelegate {
                    wei: 3_001_423,
                    is_undelegate: true,
                    nonce: 1780078264488,
                },
                "0xc24f99bd90d6d68ac3c9043c84b8c90201c000a32bd9f55c661845104fdab075",
            ),
        ] {
            assert_eq!(
                map_transaction_state_staking_action(updates.clone(), action_id, request_id.to_string()),
                TransactionUpdate::new(
                    TransactionState::Confirmed,
                    vec![TransactionChange::HashChange {
                        old: request_id.to_string(),
                        new: expected_hash.to_string(),
                    }]
                )
            );
        }
    }

    #[test]
    fn test_map_transaction_state_action_without_matching_window_stays_pending() {
        let updates = serde_json::from_str(include_str!("../../testdata/user_non_funding_ledger_updates_c_staking_transfer.json")).unwrap();
        let action_id = HyperCoreActionId::TokenDelegate {
            wei: 1_000_000,
            is_undelegate: false,
            nonce: 1779376558000,
        };
        let update = map_transaction_state_action(updates, action_id, "action:tokenDelegate:1000000:stake:1779376558000".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_prepare_perpetual_fill_maps_transaction_type() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let oid = 187530505765u64;
        let matching: Vec<_> = fills.iter().filter(|fill| fill.oid == oid).collect();
        let last_fill = matching.last().copied().unwrap();

        let (transaction_type, metadata) = prepare_perpetual_fill(&matching, last_fill).unwrap();
        assert_eq!(transaction_type, TransactionType::PerpetualOpenPosition);
        assert_eq!(metadata.direction, PerpetualDirection::Long);
        assert_eq!(metadata.is_liquidation, Some(false));
    }

    #[test]
    fn test_prepare_perpetual_fill_returns_none_for_unknown_direction() {
        let fill = UserFill {
            coin: "HYPE".to_string(),
            hash: String::new(),
            oid: 123,
            sz: "1".to_string(),
            closed_pnl: 0.0,
            fee: 0.0,
            builder_fee: None,
            fee_token: None,
            px: 42.0,
            dir: FillDirection::Other("Unsupported".to_string()),
            time: 0,
            liquidation: None,
        };

        assert!(prepare_perpetual_fill(&[&fill], &fill).is_none());
    }

    #[test]
    fn test_prepare_perpetual_fill_returns_none_for_spot_fill() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_spot_swap.json")).unwrap();
        let matching: Vec<_> = fills.iter().collect();
        let last_fill = matching.last().copied().unwrap();

        assert!(prepare_perpetual_fill(&matching, last_fill).is_none());
    }

    #[test]
    fn test_prepare_perpetual_fill_marks_liquidation() {
        let fills: Vec<UserFill> = serde_json::from_str(include_str!("../../testdata/user_fills_liquidation.json")).unwrap();
        let matching: Vec<_> = fills.iter().collect();
        let last_fill = matching.last().copied().unwrap();

        let (_, metadata) = prepare_perpetual_fill(&matching, last_fill).unwrap();
        assert_eq!(metadata.is_liquidation, Some(true));
    }
}
