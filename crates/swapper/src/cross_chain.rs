use std::collections::HashMap;

use primitives::Transaction;

use crate::SwapperProvider;

#[derive(Debug, serde::Serialize)]
pub struct VaultAddresses {
    pub deposit: Vec<String>,
    pub send: Vec<String>,
}

pub type DepositAddressMap = HashMap<String, SwapperProvider>;
pub type SendAddressMap = HashMap<String, SwapperProvider>;

pub fn swap_provider_with_vault_addresses(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> Option<SwapperProvider> {
    deposit_addresses
        .get(&transaction.to)
        .copied()
        .or_else(|| transaction.output_addresses().into_iter().find_map(|addr| deposit_addresses.get(&addr).copied()))
}

pub fn is_cross_chain_swap(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> bool {
    swap_provider_with_vault_addresses(transaction, deposit_addresses).is_some()
}

pub fn is_from_vault_address(transaction: &Transaction, send_addresses: &SendAddressMap) -> bool {
    send_addresses.contains_key(&transaction.from) || transaction.input_addresses().iter().any(|addr| send_addresses.contains_key(addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::TransactionUtxoInput;

    #[test]
    fn test_vault_address_detected() {
        let vault = "TMoD2uJiUAvB2RhLGm1BmzCVVzi5VLFDVt".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::NearIntents));
    }

    #[test]
    fn test_no_vault_address() {
        let empty = DepositAddressMap::new();
        assert!(!is_cross_chain_swap(&Transaction::mock(), &empty));
    }

    #[test]
    fn test_is_cross_chain_swap() {
        let vault = "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Thorchain)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert!(is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_utxo_vault_address_in_outputs() {
        let vault = "vault_address".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new("sender".into(), "50000".into())],
            vec![TransactionUtxoInput::new(vault, "40000".into()), TransactionUtxoInput::new("change".into(), "9000".into())],
        );
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::NearIntents));
    }

    #[test]
    fn test_utxo_no_vault_address_in_outputs() {
        let deposit_addresses = DepositAddressMap::from([("vault_address".to_string(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new("sender".into(), "50000".into())],
            vec![TransactionUtxoInput::new("recipient".into(), "40000".into())],
        );
        assert!(!is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_is_from_vault_address() {
        let vault = "vault_address".to_string();
        let send_addresses = SendAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction {
            from: vault,
            ..Transaction::mock()
        };
        assert!(is_from_vault_address(&transaction, &send_addresses));
    }

    #[test]
    fn test_is_from_vault_address_utxo() {
        let vault = "vault_address".to_string();
        let send_addresses = SendAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new(vault, "50000".into())],
            vec![TransactionUtxoInput::new("recipient".into(), "40000".into())],
        );
        assert!(is_from_vault_address(&transaction, &send_addresses));
    }

    #[test]
    fn test_is_not_from_vault_address() {
        let send_addresses = SendAddressMap::from([("vault_address".to_string(), SwapperProvider::NearIntents)]);
        assert!(!is_from_vault_address(&Transaction::mock(), &send_addresses));
    }
}
