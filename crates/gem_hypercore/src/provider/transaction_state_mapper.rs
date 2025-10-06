use crate::models::order::PerpetualFill;
use primitives::{
    PerpetualDirection, PerpetualProvider, TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionUpdate,
};

pub fn map_transaction_state_order(fills: Vec<PerpetualFill>, oid: u64, request_id: String) -> TransactionUpdate {
    let matching_fills: Vec<_> = fills.iter().filter(|fill| fill.oid == oid).collect();

    if matching_fills.is_empty() {
        return TransactionUpdate::new_state(TransactionState::Pending);
    }

    let pnl: f64 = matching_fills.iter().map(|fill| fill.closed_pnl).sum();
    let last_fill = matching_fills.last().unwrap();
    let price = last_fill.px;
    let direction = match last_fill.dir.as_str() {
        "Open Short" | "Close Short" => PerpetualDirection::Short,
        "Open Long" | "Close Long" => PerpetualDirection::Long,
        _ => PerpetualDirection::Long,
    };

    let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);
    update.changes = vec![
        TransactionChange::Metadata(TransactionMetadata::Perpetual(TransactionPerpetualMetadata {
            pnl,
            price,
            direction,
            provider: Some(PerpetualProvider::Hypercore),
        })),
        TransactionChange::HashChange {
            old: request_id,
            new: last_fill.hash.clone(),
        },
    ];
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
}
