use crate::models::order::PerpetualFill;
use primitives::{
    PerpetualDirection, PerpetualProvider, TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionUpdate,
};

fn direction_from_dir(dir: &str) -> Option<PerpetualDirection> {
    match dir {
        "Open Short" | "Close Short" => Some(PerpetualDirection::Short),
        "Open Long" | "Close Long" => Some(PerpetualDirection::Long),
        _ => None,
    }
}

fn append_perpetual_metadata(update: &mut TransactionUpdate, matching_fills: &[&PerpetualFill], last_fill: &PerpetualFill) -> bool {
    if let Some(direction) = direction_from_dir(&last_fill.dir) {
        let pnl: f64 = matching_fills.iter().map(|fill| fill.closed_pnl).sum();
        update
            .changes
            .push(TransactionChange::Metadata(TransactionMetadata::Perpetual(TransactionPerpetualMetadata {
                pnl,
                price: last_fill.px,
                direction,
                provider: Some(PerpetualProvider::Hypercore),
            })));
        return true;
    }

    false
}

pub fn map_transaction_state_order(fills: Vec<PerpetualFill>, oid: u64, request_id: String) -> TransactionUpdate {
    let matching_fills: Vec<_> = fills.iter().filter(|fill| fill.oid == oid).collect();

    if matching_fills.is_empty() {
        return TransactionUpdate::new_state(TransactionState::Pending);
    }

    let last_fill = matching_fills.last().unwrap();

    let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);

    append_perpetual_metadata(&mut update, &matching_fills, last_fill);

    if !last_fill.hash.is_empty() && last_fill.hash != request_id {
        update.changes.push(TransactionChange::HashChange {
            old: request_id,
            new: last_fill.hash.clone(),
        });
    }

    update
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::order::PerpetualFill;

    #[test]
    fn test_map_transaction_state_order() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let oid = 187530505765u64;
        let request_id = oid.to_string();

        let update = map_transaction_state_order(fills, oid, request_id.clone());

        assert_eq!(update.state, TransactionState::Confirmed);
        assert_eq!(update.changes.len(), 2);

        let has_metadata = update
            .changes
            .iter()
            .any(|change| matches!(change, TransactionChange::Metadata(TransactionMetadata::Perpetual(_))));
        assert!(has_metadata);

        let has_hash_change = update.changes.iter().any(|change| {
            matches!(
                change,
                TransactionChange::HashChange {
                    old,
                    new
                } if old == &request_id && new == "0x9b4d63110c57f2e19cc7042ce90e300202f500f6a75b11b33f160e63cb5bcccc"
            )
        });
        assert!(has_hash_change);

        for change in &update.changes {
            if let TransactionChange::Metadata(TransactionMetadata::Perpetual(metadata)) = change {
                assert_eq!(metadata.pnl, 36.5);
                assert_eq!(metadata.price, 47.904);
                assert_eq!(metadata.direction, PerpetualDirection::Long);
                assert_eq!(metadata.provider, Some(PerpetualProvider::Hypercore));
            }
        }
    }

    #[test]
    fn test_map_transaction_state_order_no_matching_fills() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let update = map_transaction_state_order(fills, 999999999u64, "999999999".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_state_order_non_perpetual_fill() {
        let fills = vec![PerpetualFill {
            coin: "HYPE".to_string(),
            hash: String::new(),
            oid: 123,
            closed_pnl: 0.0,
            fee: 0.0,
            px: 42.0,
            dir: String::new(),
        }];

        let update = map_transaction_state_order(fills, 123, "123".to_string());

        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_append_perpetual_metadata_pushes_change() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let oid = 187530505765u64;
        let matching: Vec<_> = fills.iter().filter(|fill| fill.oid == oid).collect();
        let last_fill = matching.last().copied().unwrap();

        let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);

        let added = append_perpetual_metadata(&mut update, &matching, last_fill);

        assert!(added);
        assert!(
            update
                .changes
                .iter()
                .any(|change| matches!(change, TransactionChange::Metadata(TransactionMetadata::Perpetual(_))))
        );
    }

    #[test]
    fn test_append_perpetual_metadata_returns_false_for_unknown_direction() {
        let fill = PerpetualFill {
            coin: "HYPE".to_string(),
            hash: String::new(),
            oid: 123,
            closed_pnl: 0.0,
            fee: 0.0,
            px: 42.0,
            dir: "Unsupported".to_string(),
        };

        let matching = vec![&fill];
        let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);

        let added = append_perpetual_metadata(&mut update, &matching, &fill);

        assert!(!added);
        assert!(update.changes.is_empty());
    }
}
