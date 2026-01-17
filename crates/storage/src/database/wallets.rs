use crate::models::{DeviceRow, NewWalletRow, NewWalletSubscriptionRow, WalletRow, WalletSubscriptionRow};
use crate::schema::{devices, wallets, wallets_subscriptions};
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError};
use diesel::prelude::*;
use primitives::Chain;
use std::collections::HashMap;

pub trait WalletsStore {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, DatabaseError>;
    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, DatabaseError>;
    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, DatabaseError>;
    fn create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, DatabaseError>;
    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, DatabaseError>;
    fn get_subscriptions(&mut self, device_id: &str) -> Result<Vec<(WalletRow, WalletSubscriptionRow)>, DatabaseError>;
    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, DatabaseError>;
    fn add_subscriptions(
        &mut self,
        device_id: &str,
        wallet_ids: HashMap<String, i32>,
        subscriptions: Vec<(String, Vec<(Chain, String)>)>,
    ) -> Result<usize, DatabaseError>;
    fn delete_subscriptions(
        &mut self,
        device_id: &str,
        wallet_ids: HashMap<String, i32>,
        subscriptions: Vec<(String, Vec<(Chain, String)>)>,
    ) -> Result<usize, DatabaseError>;
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
        let result = wallets::table
            .filter(wallets::id.eq(id))
            .select(WalletRow::as_select())
            .first(&mut self.connection)?;

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

    fn get_subscriptions(&mut self, device_id: &str) -> Result<Vec<(WalletRow, WalletSubscriptionRow)>, DatabaseError> {
        let results = wallets_subscriptions::table
            .inner_join(wallets::table)
            .inner_join(devices::table)
            .filter(devices::device_id.eq(device_id))
            .select((WalletRow::as_select(), WalletSubscriptionRow::as_select()))
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

    fn add_subscriptions(
        &mut self,
        device_id: &str,
        wallet_ids: HashMap<String, i32>,
        subscriptions: Vec<(String, Vec<(Chain, String)>)>,
    ) -> Result<usize, DatabaseError> {
        let device: DeviceRow = devices::table
            .filter(devices::device_id.eq(device_id))
            .select(DeviceRow::as_select())
            .first(&mut self.connection)?;

        let rows: Vec<NewWalletSubscriptionRow> = subscriptions
            .into_iter()
            .filter_map(|(wallet_identifier, addresses)| {
                wallet_ids.get(&wallet_identifier).map(|&wallet_id| {
                    addresses
                        .into_iter()
                        .map(|(chain, address)| NewWalletSubscriptionRow {
                            wallet_id,
                            device_id: device.id,
                            chain: ChainRow::from(chain),
                            address,
                        })
                        .collect::<Vec<_>>()
                })
            })
            .flatten()
            .collect();

        let count = diesel::insert_into(wallets_subscriptions::table)
            .values(&rows)
            .on_conflict((
                wallets_subscriptions::wallet_id,
                wallets_subscriptions::device_id,
                wallets_subscriptions::chain,
                wallets_subscriptions::address,
            ))
            .do_nothing()
            .execute(&mut self.connection)?;

        Ok(count)
    }

    fn delete_subscriptions(
        &mut self,
        device_id: &str,
        wallet_ids: HashMap<String, i32>,
        subscriptions: Vec<(String, Vec<(Chain, String)>)>,
    ) -> Result<usize, DatabaseError> {
        let device: DeviceRow = devices::table
            .filter(devices::device_id.eq(device_id))
            .select(DeviceRow::as_select())
            .first(&mut self.connection)?;

        let to_delete: Vec<_> = subscriptions
            .into_iter()
            .filter_map(|(wallet_identifier, addresses)| {
                wallet_ids
                    .get(&wallet_identifier)
                    .map(|&wallet_id| addresses.into_iter().map(move |(chain, address)| (wallet_id, chain, address)))
            })
            .flatten()
            .collect();

        let count = to_delete.into_iter().try_fold(0, |count, (wallet_id, chain, address)| {
            let deleted = diesel::delete(wallets_subscriptions::table)
                .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
                .filter(wallets_subscriptions::device_id.eq(device.id))
                .filter(wallets_subscriptions::chain.eq(ChainRow::from(chain)))
                .filter(wallets_subscriptions::address.eq(&address))
                .execute(&mut self.connection)?;
            Ok::<_, DatabaseError>(count + deleted)
        })?;

        Ok(count)
    }
}
