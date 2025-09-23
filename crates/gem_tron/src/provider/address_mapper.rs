use primitives::AddressStatus;

use crate::models::account::TronAccount;

pub fn map_address_status(account: &TronAccount) -> Vec<AddressStatus> {
    if let Some(owner_permission) = &account.owner_permission
        && owner_permission.permission_name != "owner"
    {
        return vec![AddressStatus::MultiSignature];
    }

    if let Some(active_permissions) = &account.active_permission
        && (active_permissions.len() > 1 || active_permissions.iter().any(|p| p.threshold > 1))
    {
        return vec![AddressStatus::MultiSignature];
    }

    vec![]
}

#[cfg(test)]
impl TronAccount {
    pub fn mock() -> Self {
        use crate::models::{TronAccountOwnerPermission, TronAccountPermission};

        TronAccount {
            balance: None,
            address: None,
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
            }),
            active_permission: Some(vec![TronAccountPermission { threshold: 1 }]),
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{TronAccount, TronAccountOwnerPermission, TronAccountPermission};

    #[test]
    fn test_map_address_status_regular_account() {
        assert!(map_address_status(&TronAccount::mock()).is_empty());
    }

    #[test]
    fn test_map_address_status_null_active_permission() {
        let mut account = TronAccount::mock();
        account.active_permission = None;
        assert!(map_address_status(&account).is_empty());
    }

    #[test]
    fn test_map_address_status_multiple_active_permissions() {
        let mut account = TronAccount::mock();
        account.active_permission = Some(vec![TronAccountPermission { threshold: 1 }, TronAccountPermission { threshold: 1 }]);
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_map_address_status_high_threshold() {
        let mut account = TronAccount::mock();
        account.active_permission = Some(vec![TronAccountPermission { threshold: 2 }]);
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_map_address_status_non_owner_permission() {
        let mut account = TronAccount::mock();
        account.owner_permission = Some(TronAccountOwnerPermission {
            permission_name: "custom".to_string(),
        });
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }
}
