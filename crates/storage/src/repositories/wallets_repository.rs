use crate::database::wallets::WalletsStore;
use crate::models::{DeviceRow, NewWalletAddressRow, NewWalletRow, NewWalletSubscriptionRow, SubscriptionAddressExcludeRow, WalletAddressRow, WalletRow, WalletSubscriptionRow};
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError};
use primitives::{Chain, DeviceSubscription};
use std::collections::{HashMap, HashSet};

pub trait WalletsRepository {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError>;
    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError>;
    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError>;
    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError>;
    fn get_or_create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError>;
    fn get_subscriptions(&mut self, device_id: i32) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)>, DatabaseError>;
    fn get_subscriptions_by_wallet_id(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<(WalletSubscriptionRow, WalletAddressRow)>, DatabaseError>;
    fn subscriptions_wallet_address_for_chain(&mut self, device_id: i32, wallet_id: i32, chain: Chain) -> Result<String, DatabaseError>;
    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, DatabaseError>;
    fn add_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError>;
    fn delete_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError>;
    fn delete_wallet_subscriptions(&mut self, device_id: i32, wallet_ids: Vec<i32>) -> Result<usize, DatabaseError>;
    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, DatabaseError>;

    fn get_subscriptions_by_chain_addresses(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError>;
    fn get_subscription_address_exists(&mut self, chain: Chain, address: &str) -> Result<bool, DatabaseError>;
    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<SubscriptionAddressExcludeRow>) -> Result<usize, DatabaseError>;
    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, DatabaseError>;
}

impl WalletsRepository for DatabaseClient {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError> {
        WalletsStore::get_wallet(self, identifier)
    }

    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError> {
        WalletsStore::get_wallet_by_id(self, id)
    }

    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError> {
        WalletsStore::get_wallets(self, identifiers)
    }

    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError> {
        WalletsStore::create_wallets(self, wallets)
    }

    fn get_or_create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError> {
        match WalletsStore::get_wallet(self, &wallet.identifier) {
            Ok(existing) => Ok(existing),
            Err(DatabaseError::NotFound) => WalletsStore::create_wallet(self, wallet),
            Err(e) => Err(e),
        }
    }

    fn get_subscriptions(&mut self, device_id: i32) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)>, DatabaseError> {
        WalletsStore::get_subscriptions_by_device_id(self, device_id)
    }

    fn get_subscriptions_by_wallet_id(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<(WalletSubscriptionRow, WalletAddressRow)>, DatabaseError> {
        WalletsStore::get_subscriptions_by_device_and_wallet(self, device_id, wallet_id)
    }

    fn subscriptions_wallet_address_for_chain(&mut self, device_id: i32, wallet_id: i32, chain: Chain) -> Result<String, DatabaseError> {
        WalletsStore::subscriptions_wallet_address_for_chain(self, device_id, wallet_id, ChainRow::from(chain))
    }

    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, DatabaseError> {
        WalletsStore::get_devices_by_wallet_id(self, wallet_id)
    }

    fn add_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        let all_addresses: Vec<String> = subscriptions.iter().map(|(_, _, addr)| addr.clone()).collect::<HashSet<_>>().into_iter().collect();

        let existing_rows = WalletsStore::get_addresses(self, all_addresses.clone())?;
        let existing_set: HashSet<String> = existing_rows.iter().map(|row| row.address.clone()).collect();

        let missing_addresses: Vec<NewWalletAddressRow> = all_addresses
            .iter()
            .filter(|addr| !existing_set.contains(*addr))
            .map(|address| NewWalletAddressRow { address: address.clone() })
            .collect();

        let new_rows = if missing_addresses.is_empty() {
            vec![]
        } else {
            let missing_strs: Vec<String> = missing_addresses.iter().map(|a| a.address.clone()).collect();
            WalletsStore::add_addresses(self, missing_addresses)?;
            WalletsStore::get_addresses(self, missing_strs)?
        };

        let address_map: HashMap<String, i32> = existing_rows.into_iter().chain(new_rows).map(|row| (row.address, row.id)).collect();

        let rows: Vec<NewWalletSubscriptionRow> = subscriptions
            .into_iter()
            .filter_map(|(wallet_id, chain, address)| {
                address_map.get(&address).map(|&address_id| NewWalletSubscriptionRow {
                    wallet_id,
                    device_id,
                    chain: ChainRow::from(chain),
                    address_id,
                })
            })
            .collect();

        if rows.is_empty() {
            return Ok(0);
        }

        WalletsStore::add_subscriptions(self, rows)
    }

    fn delete_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        let all_addresses: Vec<String> = subscriptions.iter().map(|(_, _, addr)| addr.clone()).collect::<HashSet<_>>().into_iter().collect();

        let address_rows = WalletsStore::get_addresses(self, all_addresses)?;
        let address_map: HashMap<String, i32> = address_rows.into_iter().map(|row| (row.address, row.id)).collect();

        let mut grouped: HashMap<(i32, Chain), Vec<i32>> = HashMap::new();
        for (wallet_id, chain, address) in subscriptions {
            if let Some(&address_id) = address_map.get(&address) {
                grouped.entry((wallet_id, chain)).or_default().push(address_id);
            }
        }

        if grouped.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for ((wallet_id, chain), address_ids) in grouped {
            count += WalletsStore::delete_subscriptions(self, device_id, wallet_id, ChainRow::from(chain), address_ids)?;
        }

        Ok(count)
    }

    fn delete_wallet_subscriptions(&mut self, device_id: i32, wallet_ids: Vec<i32>) -> Result<usize, DatabaseError> {
        WalletsStore::delete_wallet_subscriptions(self, device_id, wallet_ids)
    }

    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, DatabaseError> {
        WalletsStore::delete_wallet_chains(self, device_id, wallet_id, chains)
    }

    fn get_subscriptions_by_chain_addresses(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError> {
        Ok(WalletsStore::get_subscriptions_by_chain_addresses(self, chain, addresses)?
            .into_iter()
            .map(|(wallet, sub, addr, device)| DeviceSubscription {
                device: device.as_primitive(),
                wallet_id: wallet.wallet_id.0.clone(),
                chain: sub.chain.0,
                address: addr.address,
            })
            .collect())
    }

    fn get_subscription_address_exists(&mut self, chain: Chain, address: &str) -> Result<bool, DatabaseError> {
        WalletsStore::get_subscription_address_exists(self, chain, address)
    }

    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<SubscriptionAddressExcludeRow>) -> Result<usize, DatabaseError> {
        WalletsStore::add_subscriptions_exclude_addresses(self, values)
    }

    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, DatabaseError> {
        WalletsStore::get_subscriptions_exclude_addresses(self, addresses)
    }
}
