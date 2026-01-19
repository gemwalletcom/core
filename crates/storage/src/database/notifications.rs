use crate::models::{NewNotificationRow, NotificationRow};
use crate::schema::{devices, notifications, wallets, wallets_subscriptions};
use crate::{DatabaseClient, DatabaseError};
use diesel::prelude::*;

type WalletIdsSubquery<'a> = diesel::dsl::Select<
    diesel::dsl::Filter<diesel::dsl::InnerJoin<wallets_subscriptions::table, devices::table>, diesel::dsl::Eq<devices::device_id, &'a str>>,
    wallets_subscriptions::wallet_id,
>;

fn wallet_ids_by_device_id(device_id: &str) -> WalletIdsSubquery<'_> {
    wallets_subscriptions::table
        .inner_join(devices::table)
        .filter(devices::device_id.eq(device_id))
        .select(wallets_subscriptions::wallet_id)
}

pub trait NotificationsStore {
    fn get_notifications_by_device_id(&mut self, device_id: &str) -> Result<Vec<(NotificationRow, String)>, DatabaseError>;
    fn create_notifications(&mut self, notifications: Vec<NewNotificationRow>) -> Result<usize, DatabaseError>;
    fn mark_all_as_read(&mut self, device_id: &str) -> Result<usize, DatabaseError>;
}

impl NotificationsStore for DatabaseClient {
    fn get_notifications_by_device_id(&mut self, device_id: &str) -> Result<Vec<(NotificationRow, String)>, DatabaseError> {
        let results = notifications::table
            .inner_join(wallets::table)
            .filter(notifications::wallet_id.eq_any(wallet_ids_by_device_id(device_id)))
            .order(notifications::created_at.desc())
            .select((NotificationRow::as_select(), wallets::identifier))
            .load(&mut self.connection)?;

        Ok(results)
    }

    fn create_notifications(&mut self, values: Vec<NewNotificationRow>) -> Result<usize, DatabaseError> {
        Ok(diesel::insert_into(notifications::table).values(&values).execute(&mut self.connection)?)
    }

    fn mark_all_as_read(&mut self, device_id: &str) -> Result<usize, DatabaseError> {
        let count = diesel::update(notifications::table)
            .filter(notifications::wallet_id.eq_any(wallet_ids_by_device_id(device_id)))
            .filter(notifications::is_read.eq(false))
            .set((notifications::is_read.eq(true), notifications::read_at.eq(diesel::dsl::now)))
            .execute(&mut self.connection)?;

        Ok(count)
    }
}
