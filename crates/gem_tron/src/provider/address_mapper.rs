use primitives::AddressStatus;

use crate::models::account::TronAccount;

pub fn map_address_status(account: &TronAccount) -> Vec<AddressStatus> {
    let address = account.address.as_deref().unwrap_or_default();

    if let Some(owner_permission) = &account.owner_permission {
        if owner_permission.permission_name != "owner" {
            return vec![AddressStatus::MultiSignature];
        }
        if let Some(keys) = &owner_permission.keys {
            if keys.len() != 1 || keys.iter().any(|k| k.address != address) {
                return vec![AddressStatus::MultiSignature];
            }
        }
    }

    if let Some(active_permissions) = &account.active_permission {
        if active_permissions.len() > 1
            || active_permissions.iter().any(|p| p.threshold > 1)
            || active_permissions.iter().any(|p| p.id.is_some_and(|id| id != 0))
        {
            return vec![AddressStatus::MultiSignature];
        }
        for permission in active_permissions {
            if let Some(keys) = &permission.keys {
                if keys.len() != 1 || keys.iter().any(|k| k.address != address) {
                    return vec![AddressStatus::MultiSignature];
                }
            }
        }
    }

    vec![]
}

#[cfg(test)]
impl TronAccount {
    pub fn mock(address: &str) -> Self {
        use crate::models::{TronAccountOwnerPermission, TronAccountPermission, TronAccountPermissionKey};

        TronAccount {
            balance: None,
            address: Some(address.to_string()),
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
                keys: Some(vec![TronAccountPermissionKey {
                    address: address.to_string(),
                    weight: 1,
                }]),
            }),
            active_permission: Some(vec![TronAccountPermission {
                id: None,
                threshold: 1,
                keys: Some(vec![TronAccountPermissionKey {
                    address: address.to_string(),
                    weight: 1,
                }]),
            }]),
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{TronAccount, TronAccountOwnerPermission, TronAccountPermission, TronAccountPermissionKey};

    const ADDRESS: &str = "TCXbgZUdJH14fH82rf36LCpFV53dyXLY3b";
    const OTHER_ADDRESS: &str = "TW6gATnfHd4S65BB4h5Y5Wae2k93rRduLz";

    #[test]
    fn test_regular_account() {
        assert!(map_address_status(&TronAccount::mock(ADDRESS)).is_empty());
    }

    #[test]
    fn test_null_active_permission() {
        let mut account = TronAccount::mock(ADDRESS);
        account.active_permission = None;
        assert!(map_address_status(&account).is_empty());
    }

    #[test]
    fn test_multiple_active_permissions() {
        let mut account = TronAccount::mock(ADDRESS);
        account.active_permission = Some(vec![
            TronAccountPermission {
                id: None,
                threshold: 1,
                keys: None,
            },
            TronAccountPermission {
                id: None,
                threshold: 1,
                keys: None,
            },
        ]);
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_high_threshold() {
        let mut account = TronAccount::mock(ADDRESS);
        account.active_permission = Some(vec![TronAccountPermission {
            id: None,
            threshold: 2,
            keys: None,
        }]);
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_non_owner_permission_name() {
        let mut account = TronAccount::mock(ADDRESS);
        account.owner_permission = Some(TronAccountOwnerPermission {
            permission_name: "custom".to_string(),
            keys: None,
        });
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_active_permission_id_not_default() {
        let mut account = TronAccount::mock(ADDRESS);
        account.active_permission = Some(vec![TronAccountPermission {
            id: Some(2),
            threshold: 1,
            keys: None,
        }]);
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_owner_key_address_mismatch() {
        let mut account = TronAccount::mock(ADDRESS);
        account.owner_permission = Some(TronAccountOwnerPermission {
            permission_name: "owner".to_string(),
            keys: Some(vec![TronAccountPermissionKey {
                address: OTHER_ADDRESS.to_string(),
                weight: 1,
            }]),
        });
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_active_key_address_mismatch() {
        let mut account = TronAccount::mock(ADDRESS);
        account.active_permission = Some(vec![TronAccountPermission {
            id: None,
            threshold: 1,
            keys: Some(vec![TronAccountPermissionKey {
                address: OTHER_ADDRESS.to_string(),
                weight: 1,
            }]),
        }]);
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }

    #[test]
    fn test_multiple_keys() {
        let mut account = TronAccount::mock(ADDRESS);
        account.owner_permission = Some(TronAccountOwnerPermission {
            permission_name: "owner".to_string(),
            keys: Some(vec![
                TronAccountPermissionKey {
                    address: ADDRESS.to_string(),
                    weight: 1,
                },
                TronAccountPermissionKey {
                    address: OTHER_ADDRESS.to_string(),
                    weight: 1,
                },
            ]),
        });
        assert_eq!(map_address_status(&account), vec![AddressStatus::MultiSignature]);
    }
}
