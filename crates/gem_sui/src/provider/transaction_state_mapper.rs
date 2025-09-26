use primitives::{TransactionState, TransactionUpdate};

use crate::models::Digest;

pub fn map_transaction_status(transaction: Digest) -> TransactionUpdate {
    let state = match transaction.effects.status.status.as_str() {
        "success" => TransactionState::Confirmed,
        "failure" => TransactionState::Reverted,
        _ => TransactionState::Pending,
    };
    TransactionUpdate::new_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Effect, GasObject, GasUsed, Owner, Status};
    use num_bigint::BigUint;

    #[test]
    fn test_map_transaction_status() {
        let digest = Digest {
            digest: "test".to_string(),
            effects: Effect {
                gas_used: GasUsed {
                    computation_cost: BigUint::from(1000u32),
                    storage_cost: BigUint::from(500u32),
                    storage_rebate: BigUint::from(100u32),
                    non_refundable_storage_fee: BigUint::from(0u32),
                },
                status: Status { status: "success".to_string() },
                gas_object: GasObject {
                    owner: Owner::String("0x123".to_string()),
                },
            },
            balance_changes: None,
            events: vec![],
            timestamp_ms: 1234567890,
        };

        let update = map_transaction_status(digest);
        assert_eq!(update.state, TransactionState::Confirmed);
    }
}
