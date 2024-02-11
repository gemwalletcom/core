use std::error::Error;

use primitives::{
    AddressFormatter, Chain, NumberFormatter, PushNotification, PushNotificationTypes,
    Subscription, Transaction, TransactionSwapMetadata, TransactionType,
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

    pub fn get_address(&mut self, chain: Chain, address: &str) -> Result<String, Box<dyn Error>> {
        let result = self.database_client.get_scan_address(chain, address);
        match result {
            Ok(address) => Ok(address.name.unwrap_or_default()),
            Err(_) => Ok(AddressFormatter::short(chain, address)),
        }
    }

    pub fn message(
        &mut self,
        transaction: Transaction,
        subscription: Subscription,
    ) -> Result<Message, Box<dyn Error>> {
        let asset = self
            .database_client
            .get_asset(transaction.asset_id.to_string().as_str())?;
        let amount = NumberFormatter::value(transaction.value.as_str(), asset.decimals).unwrap();
        let chain = transaction.asset_id.chain;
        let to_address = self.get_address(chain, transaction.to.as_str())?;
        let from_address = self.get_address(chain, transaction.from.as_str())?;

        match transaction.transaction_type {
            TransactionType::Transfer => {
                let title = format!("Transfer {} {}", amount, asset.symbol);
                let message = if transaction
                    .input_addresses()
                    .contains(&subscription.address)
                    || transaction.from == subscription.address
                {
                    format!("To {}", to_address)
                } else {
                    format!("From {}", from_address)
                };
                Ok(Message {
                    title,
                    message: Some(message),
                })
            }
            TransactionType::TokenApproval => {
                let title = format!("Token Approval for {}", asset.symbol);
                let message = "".to_string();
                Ok(Message {
                    title,
                    message: Some(message),
                })
            }
            TransactionType::StakeDelegate => Ok(Message {
                title: format!("Stake {} {}", amount, asset.symbol),
                message: None,
            }),
            TransactionType::StakeUndelegate => Ok(Message {
                title: format!("Unstake {} {}", amount, asset.symbol),
                message: None,
            }),
            TransactionType::StakeRedelegate => Ok(Message {
                title: format!("Redelegate {} {}", amount, asset.symbol),
                message: None,
            }),
            TransactionType::StakeRewards => Ok(Message {
                title: format!("Claim Rewards {} {}", amount, asset.symbol),
                message: None,
            }),
            TransactionType::StakeWithdraw => Ok(Message {
                title: format!("Withdraw Stake {} {}", amount, asset.symbol),
                message: None,
            }),
            TransactionType::Swap => {
                let metadata: TransactionSwapMetadata =
                    serde_json::from_value(transaction.metadata)?;
                let from_asset = self
                    .database_client
                    .get_asset(metadata.from_asset.to_string().as_str())?;
                let to_asset = self
                    .database_client
                    .get_asset(metadata.to_asset.to_string().as_str())?;
                let from_amount =
                    NumberFormatter::value(metadata.from_value.as_str(), from_asset.decimals)
                        .unwrap_or_default();
                let to_amount =
                    NumberFormatter::value(metadata.to_value.as_str(), to_asset.decimals)
                        .unwrap_or_default();

                let title = format!("Swap from {} to {}", from_asset.symbol, to_asset.symbol);
                let message = format! {"{} {} > {} {}", from_amount, from_asset.symbol, to_amount, to_asset.symbol};
                Ok(Message {
                    title,
                    message: Some(message),
                })
            }
        }
    }

    pub fn get_topic(&self, platform: primitives::Platform) -> Option<String> {
        match platform {
            primitives::Platform::Android => None,
            primitives::Platform::IOS => Some(self.ios_topic.clone()),
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
            message: message.message.unwrap_or_default(),
            topic: self.get_topic(device.platform),
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
