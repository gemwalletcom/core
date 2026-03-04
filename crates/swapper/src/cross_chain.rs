use std::collections::HashMap;

use primitives::Transaction;

use crate::SwapperProvider;

pub type VaultAddressMap = HashMap<String, SwapperProvider>;

pub fn providers() -> Vec<primitives::CrossChainProvider> {
    primitives::CrossChainProvider::all()
}

pub fn swap_provider_with_vault_addresses(transaction: &Transaction, vault_addresses: &VaultAddressMap) -> Option<SwapperProvider> {
    vault_addresses.get(&transaction.to).copied()
}

pub fn is_cross_chain_swap(transaction: &Transaction, vault_addresses: &VaultAddressMap) -> bool {
    swap_provider_with_vault_addresses(transaction, vault_addresses).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_address_detected() {
        let vault = "TMoD2uJiUAvB2RhLGm1BmzCVVzi5VLFDVt".to_string();
        let vault_addresses = VaultAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &vault_addresses), Some(SwapperProvider::NearIntents));
    }

    #[test]
    fn test_no_vault_address() {
        let empty = VaultAddressMap::new();
        assert!(!is_cross_chain_swap(&Transaction::mock(), &empty));
    }

    #[test]
    fn test_is_cross_chain_swap() {
        let vault = "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string();
        let vault_addresses = VaultAddressMap::from([(vault.clone(), SwapperProvider::Thorchain)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert!(is_cross_chain_swap(&transaction, &vault_addresses));
    }
}
