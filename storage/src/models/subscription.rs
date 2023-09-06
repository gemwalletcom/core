use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Subscription {
    pub device_id: i32,
    pub chain: String,
    pub address: String,
}

impl Subscription {
    pub fn as_primitive(&self) -> primitives::Subscription {
        primitives::Subscription {
            chain: primitives::Chain::from_str(self.chain.as_str()).unwrap(),
            address: self.address.clone(),
        }
    }

    pub fn from_primitive(subscription: primitives::Subscription, device_id: i32) -> Self {
        Self {
            device_id,
            chain: subscription.chain.to_string(),
            address: subscription.address.to_string(),
        }
    }
}

