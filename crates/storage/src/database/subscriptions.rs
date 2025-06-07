use crate::schema::devices;
use crate::{models::*, DatabaseClient};
use diesel::prelude::*;

impl DatabaseClient {
    pub fn get_subscriptions_by_device_id(&mut self, _device_id: &str) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        subscriptions
            .inner_join(devices::table)
            .filter(devices::device_id.eq(_device_id))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn get_subscriptions_by_device_id_wallet_index(&mut self, _device_id: &str, _wallet_index: i32) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        subscriptions
            .filter(wallet_index.eq(_wallet_index))
            .inner_join(devices::table)
            .filter(devices::device_id.eq(_device_id))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn delete_subscription(&mut self, subscription: Subscription) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::delete(
            subscriptions
                .filter(device_id.eq(subscription.device_id))
                .filter(chain.eq(subscription.chain))
                .filter(address.eq(subscription.address)),
        )
        .execute(&mut self.connection)
    }

    pub fn get_subscriptions(&mut self, _chain: primitives::Chain, addresses: Vec<String>) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;

        let exclude_addresses = self.get_subscriptions_exclude_addresses(addresses.clone())?;

        subscriptions
            .filter(chain.eq(_chain.as_ref()))
            .filter(address.eq_any(addresses))
            .filter(address.ne_all(exclude_addresses))
            .distinct_on((device_id, chain, address))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::subscriptions_addresses_exclude::dsl::*;
        subscriptions_addresses_exclude
            .filter(address.eq_any(addresses))
            .select(address)
            .load(&mut self.connection)
    }

    pub fn add_subscriptions(&mut self, _subscriptions: Vec<Subscription>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::insert_into(subscriptions)
            .values(&_subscriptions)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_subscriptions_address_exclude(&mut self, values: Vec<SubscriptionAddressExclude>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions_addresses_exclude::dsl::*;
        diesel::insert_into(subscriptions_addresses_exclude)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
