use crate::models::{DeviceRow, NewWalletAddressRow, NewWalletRow, NewWalletSubscriptionRow, WalletAddressRow, WalletRow, WalletSubscriptionRow};
use crate::schema::{devices, wallets, wallets_addresses, wallets_subscriptions};
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError};
use diesel::prelude::*;
use primitives::Chain;

pub trait WalletsStore {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError>;
    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError>;
    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError>;
    fn create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError>;
    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError>;

    fn get_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<WalletAddressRow>, DatabaseError>;
    fn add_addresses(&mut self, addresses: Vec<NewWalletAddressRow>) -> Result<usize, DatabaseError>;

    fn get_subscriptions_by_device_id(&mut self, device_id: i32) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)>, DatabaseError>;
    fn get_subscriptions_by_device_and_wallet(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<(WalletSubscriptionRow, WalletAddressRow)>, DatabaseError>;
    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, DatabaseError>;
    fn add_subscriptions(&mut self, subscriptions: Vec<NewWalletSubscriptionRow>) -> Result<usize, DatabaseError>;
    fn delete_subscriptions(&mut self, device_id: i32, wallet_id: i32, chain: ChainRow, address_ids: Vec<i32>) -> Result<usize, DatabaseError>;
    fn delete_wallet_subscriptions(&mut self, device_id: i32, wallet_ids: Vec<i32>) -> Result<usize, DatabaseError>;
    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, DatabaseError>;
}

impl WalletsStore for DatabaseClient {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError> {
        let result = wallets::table
            .filter(wallets::identifier.eq(identifier))
            .select(WalletRow::as_select())
            .first(&mut self.connection)?;

        Ok(result)
    }

    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError> {
        let result = wallets::table.filter(wallets::id.eq(id)).select(WalletRow::as_select()).first(&mut self.connection)?;

        Ok(result)
    }

    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError> {
        let result = wallets::table
            .filter(wallets::identifier.eq_any(identifiers))
            .select(WalletRow::as_select())
            .load(&mut self.connection)?;

        Ok(result)
    }

    fn create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError> {
        let result = diesel::insert_into(wallets::table)
            .values(&wallet)
            .returning(WalletRow::as_returning())
            .get_result(&mut self.connection)?;

        Ok(result)
    }

    fn create_wallets(&mut self, new_wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError> {
        let count = diesel::insert_into(wallets::table)
            .values(&new_wallets)
            .on_conflict(wallets::identifier)
            .do_nothing()
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn get_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<WalletAddressRow>, DatabaseError> {
        let results = wallets_addresses::table
            .filter(wallets_addresses::address.eq_any(addresses))
            .select(WalletAddressRow::as_select())
            .load(&mut self.connection)?;

        Ok(results)
    }

    fn add_addresses(&mut self, addresses: Vec<NewWalletAddressRow>) -> Result<usize, DatabaseError> {
        let count = diesel::insert_into(wallets_addresses::table)
            .values(&addresses)
            .on_conflict(wallets_addresses::address)
            .do_nothing()
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn get_subscriptions_by_device_id(&mut self, device_id: i32) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)>, DatabaseError> {
        let results = wallets_subscriptions::table
            .inner_join(wallets::table)
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .select((WalletRow::as_select(), WalletSubscriptionRow::as_select(), WalletAddressRow::as_select()))
            .load(&mut self.connection)?;

        Ok(results)
    }

    fn get_subscriptions_by_device_and_wallet(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<(WalletSubscriptionRow, WalletAddressRow)>, DatabaseError> {
        let results = wallets_subscriptions::table
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .select((WalletSubscriptionRow::as_select(), WalletAddressRow::as_select()))
            .load(&mut self.connection)?;

        Ok(results)
    }

    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, DatabaseError> {
        let results = wallets_subscriptions::table
            .inner_join(devices::table)
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .select(DeviceRow::as_select())
            .distinct()
            .load(&mut self.connection)?;

        Ok(results)
    }

    fn add_subscriptions(&mut self, subscriptions: Vec<NewWalletSubscriptionRow>) -> Result<usize, DatabaseError> {
        let count = diesel::insert_into(wallets_subscriptions::table)
            .values(&subscriptions)
            .on_conflict((
                wallets_subscriptions::wallet_id,
                wallets_subscriptions::device_id,
                wallets_subscriptions::chain,
                wallets_subscriptions::address_id,
            ))
            .do_nothing()
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn delete_subscriptions(&mut self, device_id: i32, wallet_id: i32, chain: ChainRow, address_ids: Vec<i32>) -> Result<usize, DatabaseError> {
        let count = diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq(chain))
            .filter(wallets_subscriptions::address_id.eq_any(address_ids))
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn delete_wallet_subscriptions(&mut self, device_id: i32, wallet_ids: Vec<i32>) -> Result<usize, DatabaseError> {
        let count = diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq_any(wallet_ids))
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, DatabaseError> {
        let chain_rows: Vec<ChainRow> = chains.into_iter().map(ChainRow::from).collect();

        let count = diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq_any(chain_rows))
            .execute(&mut self.connection)?;

        Ok(count)
    }
}
