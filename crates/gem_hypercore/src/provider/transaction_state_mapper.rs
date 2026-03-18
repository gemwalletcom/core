use crate::models::order::PerpetualFill;
use primitives::{
    PerpetualDirection, PerpetualProvider, TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionType, TransactionUpdate,
};

fn perpetual_fill_type_and_direction(dir: &str) -> Option<(TransactionType, PerpetualDirection)> {
    match dir {
        "Open Long" => Some((TransactionType::PerpetualOpenPosition, PerpetualDirection::Long)),
        "Open Short" => Some((TransactionType::PerpetualOpenPosition, PerpetualDirection::Short)),
        "Close Long" => Some((TransactionType::PerpetualClosePosition, PerpetualDirection::Long)),
        "Close Short" => Some((TransactionType::PerpetualClosePosition, PerpetualDirection::Short)),
        _ => None,
    }
}

pub fn prepare_perpetual_fill(matching_fills: &[&PerpetualFill], last_fill: &PerpetualFill) -> Option<(TransactionType, TransactionPerpetualMetadata)> {
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

pub fn map_transaction_state_order(fills: Vec<PerpetualFill>, oid: u64, request_id: String) -> TransactionUpdate {
    let matching_fills: Vec<_> = fills.iter().filter(|fill| fill.oid == oid).collect();

    let last_fill = match matching_fills.last() {
        Some(fill) => fill,
        None => return TransactionUpdate::new_state(TransactionState::Pending),
    };

    let (_, metadata) = match prepare_perpetual_fill(&matching_fills, last_fill) {
        Some(result) => result,
        None => return TransactionUpdate::new_state(TransactionState::Pending),
    };

    let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);
    update.changes.push(TransactionChange::Metadata(TransactionMetadata::Perpetual(metadata)));

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
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
        let update = map_transaction_state_order(fills, 999999999u64, "999999999".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_state_order_non_perpetual_fill_stays_pending() {
        let fills = vec![PerpetualFill {
            coin: "HYPE".to_string(),
            hash: String::new(),
            oid: 123,
            sz: "1".to_string(),
            closed_pnl: 0.0,
            fee: 0.0,
            builder_fee: None,
            px: 42.0,
            dir: String::new(),
            time: 0,
            liquidation: None,
        }];

        let update = map_transaction_state_order(fills, 123, "123".to_string());

        assert_eq!(update.state, TransactionState::Pending);
        assert!(update.changes.is_empty());
    }

    #[test]
    fn test_prepare_perpetual_fill_maps_transaction_type() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_multiple.json")).unwrap();
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
        let fill = PerpetualFill {
            coin: "HYPE".to_string(),
            hash: String::new(),
            oid: 123,
            sz: "1".to_string(),
            closed_pnl: 0.0,
            fee: 0.0,
            builder_fee: None,
            px: 42.0,
            dir: "Unsupported".to_string(),
            time: 0,
            liquidation: None,
        };

        assert!(prepare_perpetual_fill(&[&fill], &fill).is_none());
    }

    #[test]
    fn test_prepare_perpetual_fill_marks_liquidation() {
        let fills: Vec<PerpetualFill> = serde_json::from_str(include_str!("../../testdata/user_fills_liquidation.json")).unwrap();
        let matching: Vec<_> = fills.iter().collect();
        let last_fill = matching.last().copied().unwrap();

        let (_, metadata) = prepare_perpetual_fill(&matching, last_fill).unwrap();
        assert_eq!(metadata.is_liquidation, Some(true));
    }
}
