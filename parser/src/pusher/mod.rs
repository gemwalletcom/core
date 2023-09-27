pub mod model;
pub mod client;

use std::error::Error;

use primitives::{Transaction, Subscription, AddressFormatter};
use rust_decimal::{Decimal, prelude::*};
use storage::DatabaseClient;

use self::client::PusherClient;
use self::model::{Notifications, Notification};

pub struct Pusher {
    ios_topic: String,
    client: PusherClient,
    database_client: DatabaseClient,
}

impl Pusher {
    pub fn new(
        url: String,
        database_url: String,
        ios_topic: String,
    ) -> Self {
        let client = PusherClient::new(url.clone());
        let database_client = DatabaseClient::new(&database_url);
        Self {
            ios_topic,
            client,
            database_client,
        }
    }

    pub async fn push(&mut self, device: primitives::Device, transaction: Transaction, subscription: Subscription) -> Result<usize, Box<dyn Error>> {
        // only push if push is enabled and token is set
        if !device.is_push_enabled || device.token.is_empty() {
            return Ok(0)
        }

        let asset = self.database_client.get_asset(transaction.asset_id.to_string())?;
        let mut crypto_amount: Decimal = Decimal::from_str(transaction.value.as_str())?;
        crypto_amount.set_scale(asset.decimals as u32).unwrap_or_default();
        let amount = crypto_amount.to_f64().unwrap_or_default();

        let title = format!("Transfer {} {}", amount, asset.symbol);
        let message = if transaction.from == subscription.address { 
            format!("To {}", AddressFormatter::short(transaction.asset_id.chain, transaction.to.as_str())) 
        } else {
            format!("From {}", AddressFormatter::short(transaction.asset_id.chain, transaction.from.as_str())) 
        };

        let notification = Notification {
            tokens: vec![device.token],
            platform: device.platform.as_i32(),
            title,
            message,
            topic: self.ios_topic.clone(),
        };
        let response = self.client.push(notification).await?;

        if response.logs.len() > 0 {
            println!("push logs: {:?}", response.logs);
            let _ = self.database_client.update_device_is_push_enabled(&device.id, false)?;
        }

        return Ok(response.counts as usize);
    }
}