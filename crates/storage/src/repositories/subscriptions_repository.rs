use std::error::Error;

use crate::database::subscriptions::SubscriptionsStore;
use crate::DatabaseClient;
use primitives::{Chain, DeviceSubscription, Subscription as PrimitiveSubscription};

pub trait SubscriptionsRepository {
    fn get_subscriptions_by_device_id(
        &mut self,
        device_id: &str,
        wallet_index: Option<i32>,
    ) -> Result<Vec<PrimitiveSubscription>, Box<dyn Error + Send + Sync>>;
    fn get_subscriptions(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, Box<dyn Error + Send + Sync>>;
    fn add_subscriptions(&mut self, values: Vec<PrimitiveSubscription>, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn delete_subscriptions(&mut self, values: Vec<PrimitiveSubscription>, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn delete_subscriptions_for_device_ids(&mut self, device_ids: Vec<i32>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<crate::models::SubscriptionAddressExclude>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl SubscriptionsRepository for DatabaseClient {
    fn get_subscriptions_by_device_id(
        &mut self,
        device_id: &str,
        wallet_index: Option<i32>,
    ) -> Result<Vec<PrimitiveSubscription>, Box<dyn Error + Send + Sync>> {
        Ok(SubscriptionsStore::get_subscriptions_by_device_id(self, device_id, wallet_index)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }

    fn delete_subscriptions(&mut self, values: Vec<PrimitiveSubscription>, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        use crate::database::devices::DevicesStore;
        let device = DevicesStore::get_device(self, device_id)?;
        Ok(SubscriptionsStore::delete_subscriptions(
            self,
            values.into_iter().map(|x| crate::models::Subscription::from_primitive(x, device.id)).collect(),
        )?)
    }

    fn get_subscriptions(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, Box<dyn Error + Send + Sync>> {
        Ok(SubscriptionsStore::get_subscriptions_with_device(self, chain, addresses)?
            .into_iter()
            .map(|(subscription, device)| DeviceSubscription {
                device: device.as_primitive(),
                subscription: subscription.as_primitive(),
            })
            .collect())
    }

    fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(SubscriptionsStore::get_subscriptions_exclude_addresses(self, addresses)?)
    }

    fn add_subscriptions(&mut self, values: Vec<PrimitiveSubscription>, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        use crate::database::devices::DevicesStore;
        let device = DevicesStore::get_device(self, device_id)?;
        Ok(SubscriptionsStore::add_subscriptions(
            self,
            values.into_iter().map(|x| crate::models::Subscription::from_primitive(x, device.id)).collect(),
        )?)
    }

    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<crate::models::SubscriptionAddressExclude>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(SubscriptionsStore::add_subscriptions_exclude_addresses(self, values)?)
    }

    fn delete_subscriptions_for_device_ids(&mut self, device_ids: Vec<i32>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(SubscriptionsStore::delete_subscriptions_for_device_ids(self, device_ids)?)
    }
}
