use crate::rpc::model::TransactionReciept;
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

pub fn map_transaction_broadcast(data: &str) -> String {
    if data.starts_with("0x") { data.to_string() } else { format!("0x{}", data) }
}

pub fn map_transaction_status(receipt: &TransactionReciept) -> TransactionUpdate {
    if receipt.status == "0x0" || receipt.status == "0x1" {
        let state = match receipt.status.as_str() {
            "0x0" => TransactionState::Reverted,
            "0x1" => TransactionState::Confirmed,
            _ => TransactionState::Confirmed,
        };

        let network_fee: BigInt = receipt.get_fee().into();

        TransactionUpdate::new(state, vec![TransactionChange::NetworkFee(network_fee)])
    } else {
        TransactionUpdate::new_state(TransactionState::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn map_transaction_broadcast_encode() {
        assert_eq!(map_transaction_broadcast("123"), "0x123");
        assert_eq!(map_transaction_broadcast("0x123"), "0x123");
    }

    #[test]
    fn map_transaction_status_confirmed() {
        let receipt = TransactionReciept {
            gas_used: BigUint::from(21000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_number: BigUint::from(0x123u32),
        };

        let result = map_transaction_status(&receipt);

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(420000000000000u64))]);
    }

    #[test]
    fn map_transaction_status_reverted() {
        let receipt = TransactionReciept {
            gas_used: BigUint::from(21000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x0".to_string(),
            block_number: BigUint::from(0x123u32),
        };

        let result = map_transaction_status(&receipt);

        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(420000000000000u64))]);
    }

    #[test]
    fn map_transaction_status_unknown_as_pending() {
        let receipt = TransactionReciept {
            gas_used: BigUint::from(21000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x2".to_string(),
            block_number: BigUint::from(0x123u32),
        };

        let result = map_transaction_status(&receipt);

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);
    }

    #[test]
    fn map_transaction_status_confirmed_with_l1_fee() {
        let receipt = TransactionReciept {
            gas_used: BigUint::from(21000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: Some(BigUint::from(5000000000000000u64)),
            logs: vec![],
            status: "0x1".to_string(),
            block_number: BigUint::from(0x123u32),
        };

        let result = map_transaction_status(&receipt);

        assert_eq!(result.state, TransactionState::Confirmed);
        let expected_total = BigInt::from(21000u32) * BigInt::from(20000000000u64) + BigInt::from(5000000000000000u64);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(expected_total)]);
    }
}
