use std::str::FromStr;

use diesel::prelude::*;
use primitives::{Chain, ChainAddress, Subscription};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubscriptionRow {
    pub device_id: i32,
    pub wallet_index: i32,
    pub chain: String,
    pub address: String,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::subscriptions_addresses_exclude)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubscriptionAddressExcludeRow {
    pub address: String,
    pub chain: String,
}

impl SubscriptionRow {
    pub fn as_primitive(&self) -> Subscription {
        Subscription {
            wallet_index: self.wallet_index,
            chain: Chain::from_str(self.chain.as_ref()).unwrap(),
            address: self.address.clone(),
        }
    }

    pub fn as_chain_address(&self) -> ChainAddress {
        ChainAddress {
            chain: Chain::from_str(self.chain.as_ref()).unwrap(),
            address: self.address.clone(),
        }
    }

    pub fn from_primitive(subscription: Subscription, device_id: i32, wallet_index: i32) -> Self {
        Self {
            device_id,
            wallet_index,
            chain: subscription.chain.as_ref().to_string(),
            address: subscription.address.to_string(),
        }
    }
}
