use std::error::Error;

use localizer::LanguageLocalizer;
use number_formatter::BigNumberFormatter;
use primitives::{
    AddressFormatter, Asset, Chain, GorushNotification, PushNotification, PushNotificationTransaction, PushNotificationTypes, Subscription, Transaction,
    TransactionSwapMetadata, TransactionType,
};
use storage::DatabaseClient;

use api_connector::pusher::model::Message;

pub struct Pusher {
    database_client: DatabaseClient,
}

impl Pusher {
    pub fn new(database_url: &str) -> Self {
        let database_client = DatabaseClient::new(database_url);
        Self { database_client }
    }

    pub fn get_address(&mut self, chain: Chain, address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let result = self.database_client.get_scan_address(chain, address);
        match result {
            Ok(address) => Ok(address.name.unwrap_or_default()),
            Err(_) => Ok(AddressFormatter::short(chain, address)),
        }
    }

    pub fn message(
        &mut self,
        localizer: LanguageLocalizer,
        transaction: Transaction,
        subscription: Subscription,
        assets: Vec<Asset>,
    ) -> Result<Message, Box<dyn Error + Send + Sync>> {
        let asset = assets.iter().find(|x| x.id == transaction.asset_id).ok_or("Asset not found")?;
        let amount = BigNumberFormatter::value(transaction.value.as_str(), asset.decimals).unwrap_or_default();
        let chain = transaction.asset_id.chain;

        let to_address = self.get_address(chain, transaction.to.as_str())?;
        let from_address = self.get_address(chain, transaction.from.as_str())?;

        match transaction.transaction_type {
            TransactionType::Transfer | TransactionType::TransferNFT | TransactionType::SmartContractCall => {
                let is_sent = transaction.input_addresses().contains(&subscription.address) || transaction.from == subscription.address;

                let title = localizer.notification_transfer_title(is_sent, self.get_value(amount, asset.symbol.clone()).as_str());

                let message = localizer.notification_transfer_description(is_sent, to_address.as_str(), from_address.as_str());

                Ok(Message { title, message: Some(message) })
            }
            TransactionType::TokenApproval => Ok(Message {
                title: localizer.notification_token_approval_title(asset.symbol.as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeDelegate => Ok(Message {
                title: localizer.notification_stake_title(self.get_value(amount, asset.symbol.clone()).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeUndelegate => Ok(Message {
                title: localizer.notification_unstake_title(self.get_value(amount, asset.symbol.clone()).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeRedelegate => Ok(Message {
                title: localizer.notification_redelegate_title(self.get_value(amount, asset.symbol.clone()).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeRewards => Ok(Message {
                title: localizer.notification_claim_rewards_title(self.get_value(amount, asset.symbol.clone()).as_str()),
                message: None,
            }),
            TransactionType::StakeWithdraw => Ok(Message {
                title: localizer.notification_withdraw_stake_title(self.get_value(amount, asset.symbol.clone()).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::Swap => {
                let metadata = transaction.metadata.ok_or("Missing metadata")?;
                let metadata: TransactionSwapMetadata = serde_json::from_value(metadata)?;

                let from_asset = assets
                    .iter()
                    .find(|x| x.id == metadata.from_asset)
                    .ok_or(format!("Asset not found: {}", metadata.from_asset))?;
                let to_asset = assets
                    .iter()
                    .find(|x| x.id == metadata.to_asset)
                    .ok_or(format!("Asset not found: {}", metadata.to_asset))?;

                let from_amount = BigNumberFormatter::value(metadata.from_value.as_str(), from_asset.decimals).unwrap_or_default();
                let to_amount = BigNumberFormatter::value(metadata.to_value.as_str(), to_asset.decimals).unwrap_or_default();

                Ok(Message {
                    title: localizer.notification_swap_title(from_asset.symbol.as_str(), to_asset.symbol.as_str()),
                    message: Some(localizer.notification_swap_description(
                        self.get_value(from_amount, from_asset.symbol.clone()).as_str(),
                        self.get_value(to_amount, to_asset.symbol.clone()).as_str(),
                    )),
                })
            }
            TransactionType::AssetActivation => todo!(),
        }
    }

    pub fn get_value(&self, amount: String, symbol: String) -> String {
        format! {"{} {}", amount, symbol}
    }

    pub async fn get_messages(
        &mut self,
        device: primitives::Device,
        transaction: Transaction,
        subscription: Subscription,
        assets: Vec<Asset>,
    ) -> Result<Vec<GorushNotification>, Box<dyn Error + Send + Sync>> {
        // only push if push is enabled and token is set
        if !device.is_push_enabled || device.token.is_empty() {
            return Ok(vec![]);
        }
        let localizer = LanguageLocalizer::new_with_language(&device.locale);
        let message = self.message(localizer, transaction.clone(), subscription.clone(), assets.clone())?;

        let notification_transaction = PushNotificationTransaction {
            wallet_index: subscription.wallet_index,
            transaction_id: transaction.id,
            asset_id: transaction.asset_id.to_string(),
        };
        let data = PushNotification {
            notification_type: PushNotificationTypes::Transaction,
            data: serde_json::to_value(&notification_transaction).ok(),
        };

        let notification = GorushNotification::new(
            vec![device.token],
            device.platform.as_i32(),
            message.title,
            message.message.unwrap_or_default(),
            data,
        );

        Ok(vec![notification])
    }
}
