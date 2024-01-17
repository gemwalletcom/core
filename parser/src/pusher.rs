use std::error::Error;

use primitives::{
    AddressFormatter, NumberFormatter, PushNotification, PushNotificationTypes, Subscription,
    Transaction, TransactionSwapMetadata, TransactionType,
};
use storage::DatabaseClient;

use api_connector::pusher::model::{Message, Notification};
use api_connector::PusherClient;

pub struct Pusher {
    ios_topic: String,
    client: PusherClient,
    database_client: DatabaseClient,
}

impl Pusher {
    pub fn new(url: String, database_url: String, ios_topic: String) -> Self {
        let client = PusherClient::new(url.clone());
        let database_client = DatabaseClient::new(&database_url);
        Self {
            ios_topic,
            client,
            database_client,
        }
    }

    pub fn message(
        &mut self,
        transaction: Transaction,
        subscription: Subscription,
    ) -> Result<Message, Box<dyn Error>> {
        let asset = self
            .database_client
            .get_asset(transaction.asset_id.to_string())?;

        match transaction.transaction_type {
            TransactionType::Transfer => {
                let amount =
                    NumberFormatter::value(transaction.value.as_str(), asset.decimals).unwrap();
                let title = format!("Transfer {} {}", amount, asset.symbol);
                let message = if transaction
                    .input_addresses()
                    .contains(&subscription.address)
                    || transaction.from == subscription.address
                {
                    format!(
                        "To {}",
                        AddressFormatter::short(
                            transaction.asset_id.chain,
                            transaction.to.as_str()
                        )
                    )
                } else {
                    format!(
                        "From {}",
                        AddressFormatter::short(
                            transaction.asset_id.chain,
                            transaction.from.as_str()
                        )
                    )
                };
                Ok(Message { title, message })
            }
            TransactionType::TokenApproval => {
                let title = format!("Token Approval for {}", asset.symbol);
                let message = "".to_string();
                Ok(Message { title, message })
            }
            TransactionType::StakeDelegate => Ok(Message {
                title: format!("Stake {}", asset.symbol),
                message: "".to_string(),
            }),
            TransactionType::StakeUndelegate => Ok(Message {
                title: format!("Unstake {}", asset.symbol),
                message: "".to_string(),
            }),
            TransactionType::StakeRedelegate => Ok(Message {
                title: format!("Redelegate {}", asset.symbol),
                message: "".to_string(),
            }),
            TransactionType::StakeRewards => Ok(Message {
                title: format!("Claim Rewards {}", asset.symbol),
                message: "".to_string(),
            }),
            TransactionType::Swap => {
                let metadata: TransactionSwapMetadata =
                    serde_json::from_value(transaction.metadata)?;
                let from_asset = self
                    .database_client
                    .get_asset(metadata.from_asset.to_string())?;
                let to_asset = self
                    .database_client
                    .get_asset(metadata.to_asset.to_string())?;
                let from_amount =
                    NumberFormatter::value(metadata.from_value.as_str(), from_asset.decimals)
                        .unwrap_or_default();
                let to_amount =
                    NumberFormatter::value(metadata.to_value.as_str(), to_asset.decimals)
                        .unwrap_or_default();

                let title = format!("Swap from {} to {}", from_asset.symbol, to_asset.symbol);
                let message = format! {"{} {} > {} {}", from_amount, from_asset.symbol, to_amount, to_asset.symbol};
                Ok(Message { title, message })
            }
        }
    }

    pub async fn push(
        &mut self,
        device: primitives::Device,
        transaction: Transaction,
        subscription: Subscription,
    ) -> Result<usize, Box<dyn Error>> {
        // only push if push is enabled and token is set
        if !device.is_push_enabled || device.token.is_empty() {
            return Ok(0);
        }
        let message = self.message(transaction.clone(), subscription.clone())?;
        let data = PushNotification {
            notification_type: PushNotificationTypes::Transaction,
            data: transaction,
        };

        let notification = Notification {
            tokens: vec![device.token],
            platform: device.platform.as_i32(),
            title: message.title,
            message: message.message,
            topic: self.ios_topic.clone(),
            data: Some(data),
        };
        let response = self.client.push(notification).await?;

        if !response.logs.is_empty() {
            println!("push logs: {:?}", response.logs);
            let _ = self
                .database_client
                .update_device_is_push_enabled(&device.id, false)?;
        }

        Ok(response.counts as usize)
    }
}
