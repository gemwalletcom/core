use crate::DatabaseClient;
use crate::models::{DeviceRow, NewWalletAddressRow, NewWalletRow, NewWalletSubscriptionRow, SubscriptionAddressExcludeRow, WalletAddressRow, WalletRow, WalletSubscriptionRow};
use crate::schema::{devices, subscriptions_addresses_exclude, wallets, wallets_addresses, wallets_subscriptions};
use crate::sql_types::ChainRow;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::Chain;

pub trait WalletsStore {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, diesel::result::Error>;
    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, diesel::result::Error>;
    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, diesel::result::Error>;
    fn create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, diesel::result::Error>;
    fn create_wallets(&mut self, wallets: Vec<NewWalletRow>) -> Result<usize, diesel::result::Error>;

    fn get_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<WalletAddressRow>, diesel::result::Error>;
    fn add_addresses(&mut self, addresses: Vec<NewWalletAddressRow>) -> Result<usize, diesel::result::Error>;

    fn get_subscriptions_by_device_id(&mut self, device_id: i32) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)>, diesel::result::Error>;
    fn get_subscriptions_by_device_and_wallet(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<(WalletSubscriptionRow, WalletAddressRow)>, diesel::result::Error>;
    fn subscriptions_wallet_address_for_chain(&mut self, device_id: i32, wallet_id: i32, chain: ChainRow) -> Result<WalletAddressRow, diesel::result::Error>;
    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, diesel::result::Error>;
    fn add_subscriptions(&mut self, subscriptions: Vec<NewWalletSubscriptionRow>) -> Result<usize, diesel::result::Error>;
    fn delete_subscriptions(&mut self, device_id: i32, wallet_id: i32, chain: ChainRow, address_ids: Vec<i32>) -> Result<usize, diesel::result::Error>;
    fn delete_wallet_subscriptions(&mut self, device_id: i32, wallet_ids: Vec<i32>) -> Result<usize, diesel::result::Error>;
    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, diesel::result::Error>;
    fn delete_subscriptions_for_device_ids(&mut self, device_ids: Vec<i32>) -> Result<usize, diesel::result::Error>;

    fn get_subscriptions_by_chain_addresses(
        &mut self,
        chain: Chain,
        addresses: Vec<String>,
    ) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow, DeviceRow)>, diesel::result::Error>;
    fn get_subscription_address_exists(&mut self, chain: Chain, address: &str) -> Result<bool, diesel::result::Error>;
    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<SubscriptionAddressExcludeRow>) -> Result<usize, diesel::result::Error>;
    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, diesel::result::Error>;
    fn get_device_addresses(&mut self, device_id: i32, chain: ChainRow) -> Result<Vec<String>, diesel::result::Error>;
    fn get_first_subscription_date_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<NaiveDateTime>, diesel::result::Error>;
}

impl WalletsStore for DatabaseClient {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRow, diesel::result::Error> {
        wallets::table
            .filter(wallets::identifier.eq(identifier))
            .select(WalletRow::as_select())
            .first(&mut self.connection)
    }

    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRow, diesel::result::Error> {
        wallets::table.filter(wallets::id.eq(id)).select(WalletRow::as_select()).first(&mut self.connection)
    }

    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRow>, diesel::result::Error> {
        wallets::table
            .filter(wallets::identifier.eq_any(identifiers))
            .select(WalletRow::as_select())
            .load(&mut self.connection)
    }

    fn create_wallet(&mut self, wallet: NewWalletRow) -> Result<WalletRow, diesel::result::Error> {
        diesel::insert_into(wallets::table)
            .values(&wallet)
            .returning(WalletRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn create_wallets(&mut self, new_wallets: Vec<NewWalletRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(wallets::table)
            .values(&new_wallets)
            .on_conflict(wallets::identifier)
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn get_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<WalletAddressRow>, diesel::result::Error> {
        wallets_addresses::table
            .filter(wallets_addresses::address.eq_any(addresses))
            .select(WalletAddressRow::as_select())
            .load(&mut self.connection)
    }

    fn add_addresses(&mut self, addresses: Vec<NewWalletAddressRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(wallets_addresses::table)
            .values(&addresses)
            .on_conflict(wallets_addresses::address)
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn get_subscriptions_by_device_id(&mut self, device_id: i32) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)>, diesel::result::Error> {
        wallets_subscriptions::table
            .inner_join(wallets::table)
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .select((WalletRow::as_select(), WalletSubscriptionRow::as_select(), WalletAddressRow::as_select()))
            .load(&mut self.connection)
    }

    fn get_subscriptions_by_device_and_wallet(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<(WalletSubscriptionRow, WalletAddressRow)>, diesel::result::Error> {
        wallets_subscriptions::table
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .select((WalletSubscriptionRow::as_select(), WalletAddressRow::as_select()))
            .load(&mut self.connection)
    }

    fn subscriptions_wallet_address_for_chain(&mut self, device_id: i32, wallet_id: i32, chain: ChainRow) -> Result<WalletAddressRow, diesel::result::Error> {
        wallets_subscriptions::table
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq(chain))
            .select(WalletAddressRow::as_select())
            .first(&mut self.connection)
    }

    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<DeviceRow>, diesel::result::Error> {
        wallets_subscriptions::table
            .inner_join(devices::table)
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .select(DeviceRow::as_select())
            .distinct()
            .load(&mut self.connection)
    }

    fn add_subscriptions(&mut self, subscriptions: Vec<NewWalletSubscriptionRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(wallets_subscriptions::table)
            .values(&subscriptions)
            .on_conflict((
                wallets_subscriptions::wallet_id,
                wallets_subscriptions::device_id,
                wallets_subscriptions::chain,
                wallets_subscriptions::address_id,
            ))
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn delete_subscriptions(&mut self, device_id: i32, wallet_id: i32, chain: ChainRow, address_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
        diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq(chain))
            .filter(wallets_subscriptions::address_id.eq_any(address_ids))
            .execute(&mut self.connection)
    }

    fn delete_wallet_subscriptions(&mut self, device_id: i32, wallet_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
        diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq_any(wallet_ids))
            .execute(&mut self.connection)
    }

    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, diesel::result::Error> {
        let chain_rows: Vec<ChainRow> = chains.into_iter().map(ChainRow::from).collect();

        diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq_any(chain_rows))
            .execute(&mut self.connection)
    }

    fn delete_subscriptions_for_device_ids(&mut self, device_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
        diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq_any(device_ids))
            .execute(&mut self.connection)
    }

    fn get_subscriptions_by_chain_addresses(
        &mut self,
        chain: Chain,
        addresses: Vec<String>,
    ) -> Result<Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow, DeviceRow)>, diesel::result::Error> {
        let chain_row = ChainRow::from(chain);

        wallets_subscriptions::table
            .inner_join(wallets::table)
            .inner_join(wallets_addresses::table)
            .inner_join(devices::table)
            .filter(wallets_subscriptions::chain.eq(chain_row))
            .filter(wallets_addresses::address.eq_any(&addresses))
            .filter(diesel::dsl::not(diesel::dsl::exists(
                subscriptions_addresses_exclude::table.filter(subscriptions_addresses_exclude::address.eq(wallets_addresses::address)),
            )))
            .select((
                WalletRow::as_select(),
                WalletSubscriptionRow::as_select(),
                WalletAddressRow::as_select(),
                DeviceRow::as_select(),
            ))
            .load(&mut self.connection)
    }

    fn get_subscription_address_exists(&mut self, chain: Chain, address: &str) -> Result<bool, diesel::result::Error> {
        let chain_row = ChainRow::from(chain);

        diesel::select(diesel::dsl::exists(
            wallets_subscriptions::table
                .inner_join(wallets_addresses::table)
                .filter(wallets_subscriptions::chain.eq(chain_row))
                .filter(wallets_addresses::address.eq(address)),
        ))
        .get_result(&mut self.connection)
    }

    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<SubscriptionAddressExcludeRow>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(subscriptions_addresses_exclude::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, diesel::result::Error> {
        subscriptions_addresses_exclude::table
            .filter(subscriptions_addresses_exclude::address.eq_any(addresses))
            .select(subscriptions_addresses_exclude::address)
            .load(&mut self.connection)
    }

    fn get_device_addresses(&mut self, device_id: i32, chain: ChainRow) -> Result<Vec<String>, diesel::result::Error> {
        wallets_subscriptions::table
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::chain.eq(chain))
            .select(wallets_addresses::address)
            .load(&mut self.connection)
    }

    fn get_first_subscription_date_by_wallet_id(&mut self, wallet_id: i32) -> Result<Option<NaiveDateTime>, diesel::result::Error> {
        wallets_subscriptions::table
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .select(wallets_subscriptions::created_at)
            .order(wallets_subscriptions::created_at.asc())
            .first(&mut self.connection)
            .optional()
    }
}
