use crate::schema::devices;
use crate::{models::*, DatabaseClient};
use diesel::prelude::*;
use primitives::Chain;

pub(crate) trait SubscriptionsStore {
    fn get_subscriptions_by_device_id(&mut self, device_id: &str, wallet_index: Option<i32>) -> Result<Vec<Subscription>, diesel::result::Error>;
    fn get_subscriptions(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<(Subscription, Device)>, diesel::result::Error>;
    fn add_subscriptions(&mut self, values: Vec<Subscription>) -> Result<usize, diesel::result::Error>;
    fn delete_subscriptions(&mut self, values: Vec<Subscription>) -> Result<usize, diesel::result::Error>;
    fn delete_subscriptions_for_device_ids(&mut self, device_ids: Vec<i32>) -> Result<usize, diesel::result::Error>;
    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, diesel::result::Error>;
    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<SubscriptionAddressExclude>) -> Result<usize, diesel::result::Error>;
}

impl SubscriptionsStore for DatabaseClient {
    fn get_subscriptions_by_device_id(&mut self, _device_id: &str, _wallet_index: Option<i32>) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;

        let mut query = subscriptions.inner_join(devices::table).filter(devices::device_id.eq(_device_id)).into_boxed();

        if let Some(index) = _wallet_index {
            query = query.filter(wallet_index.eq(index));
        }

        query.select(Subscription::as_select()).load(&mut self.connection)
    }

    fn delete_subscriptions(&mut self, values: Vec<Subscription>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        let mut result = 0;
        for subscription in values {
            result += diesel::delete(
                subscriptions
                    .filter(device_id.eq(subscription.device_id))
                    .filter(chain.eq(subscription.chain))
                    .filter(address.eq(subscription.address)),
            )
            .execute(&mut self.connection)?;
        }
        Ok(result)
    }

    fn get_subscriptions(&mut self, _chain: Chain, addresses: Vec<String>) -> Result<Vec<(Subscription, Device)>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        use crate::schema::subscriptions_addresses_exclude;

        subscriptions
            .inner_join(devices::table)
            .filter(chain.eq(_chain.as_ref()))
            .filter(address.eq_any(addresses))
            .filter(diesel::dsl::not(diesel::dsl::exists(
                subscriptions_addresses_exclude::table.filter(subscriptions_addresses_exclude::address.eq(address)),
            )))
            .distinct_on((device_id, chain, address))
            .select((Subscription::as_select(), crate::models::Device::as_select()))
            .load(&mut self.connection)
    }

    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::subscriptions_addresses_exclude::dsl::*;
        subscriptions_addresses_exclude
            .filter(address.eq_any(addresses))
            .select(address)
            .load(&mut self.connection)
    }

    fn add_subscriptions(&mut self, values: Vec<Subscription>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::insert_into(subscriptions)
            .values(&values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<SubscriptionAddressExclude>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions_addresses_exclude::dsl::*;
        diesel::insert_into(subscriptions_addresses_exclude)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn delete_subscriptions_for_device_ids(&mut self, device_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::delete(subscriptions.filter(device_id.eq_any(device_ids))).execute(&mut self.connection)
    }
}
