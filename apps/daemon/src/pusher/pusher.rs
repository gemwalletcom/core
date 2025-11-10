use std::error::Error;

use localizer::LanguageLocalizer;
use number_formatter::BigNumberFormatter;
use primitives::{
    AddressFormatter, Asset, AssetVecExt, Chain, GorushNotification, NFTAssetId, PushNotification, PushNotificationTransaction, PushNotificationTypes,
    Subscription, Transaction, TransactionNFTTransferMetadata, TransactionSwapMetadata, TransactionType,
};
use storage::Database;

use api_connector::pusher::model::Message;

pub struct Pusher {
    database: Database,
}

impl Pusher {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_address(&mut self, chain: Chain, address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let result = self.database.client()?.scan_addresses().get_scan_address(chain, address);
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
        let asset = assets.asset_result(transaction.asset_id.clone())?;
        let amount = BigNumberFormatter::value(transaction.value.as_str(), asset.decimals).unwrap_or_default();
        let chain = transaction.asset_id.chain;

        let to_address = self.get_address(chain, transaction.to.as_str())?;
        let from_address = self.get_address(chain, transaction.from.as_str())?;

        match transaction.transaction_type {
            TransactionType::Transfer | TransactionType::SmartContractCall => {
                let is_sent = transaction.is_sent(subscription.address.clone());
                let value = self.get_value(amount, asset.symbol.clone());
                let title = localizer.notification_transfer_title(is_sent, value.as_str());
                let message = localizer.notification_transfer_description(is_sent, to_address.as_str(), from_address.as_str());
                Ok(Message { title, message: Some(message) })
            }
            TransactionType::TransferNFT => {
                let metadata = transaction.clone().metadata.ok_or("Missing metadata")?;
                let metadata: TransactionNFTTransferMetadata = serde_json::from_value(metadata)?;
                let nft_asset_id = NFTAssetId::from_id(&metadata.asset_id.clone()).ok_or("Missing nft asset id")?;
                let name = if let Some(name) = metadata.name {
                    name
                } else if nft_asset_id.token_id.len() < 6 {
                    format!("#{}", nft_asset_id.token_id)
                } else {
                    format!("#{}...", nft_asset_id.token_id.get(..6).unwrap_or(&nft_asset_id.token_id))
                };
                let is_sent = transaction.is_sent(subscription.address.clone());
                let title = localizer.notification_nft_transfer_title(is_sent, &name);
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
                let from_asset = assets.asset_result(metadata.from_asset.clone())?;
                let to_asset = assets.asset_result(metadata.to_asset.clone())?;
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
            TransactionType::PerpetualOpenPosition => {
                let _is_sent = transaction.is_sent(subscription.address.clone());
                let value = self.get_value(amount, asset.symbol.clone());
                let title = format!("Opened Perpetual Position: {value}");
                let message = format!("Opened perpetual position for {value} at {to_address}");
                Ok(Message { title, message: Some(message) })
            }
            TransactionType::PerpetualClosePosition => {
                let _is_sent = transaction.is_sent(subscription.address.clone());
                let value = self.get_value(amount, asset.symbol.clone());
                let title = format!("Closed Perpetual Position: {value}");
                let message = format!("Closed perpetual position for {value} at {to_address}");
                Ok(Message { title, message: Some(message) })
            }
            TransactionType::AssetActivation | TransactionType::PerpetualModifyPosition => todo!(),
            TransactionType::StakeFreeze => Ok(Message {
                title: localizer.notification_freeze_title(self.get_value(amount, asset.symbol.clone()).as_str()),
                message: None,
            }),
            TransactionType::StakeUnfreeze => Ok(Message {
                title: localizer.notification_unfreeze_title(self.get_value(amount, asset.symbol.clone()).as_str()),
                message: None,
            }),
        }
    }

    pub fn get_value(&self, amount: String, symbol: String) -> String {
        format! {"{amount} {symbol}"}
    }

    pub async fn get_messages(
        &mut self,
        device: primitives::Device,
        transaction: Transaction,
        subscription: Subscription,
        assets: Vec<Asset>,
    ) -> Result<Vec<GorushNotification>, Box<dyn Error + Send + Sync>> {
        if !device.can_receive_push_notification() {
            return Ok(vec![]);
        }
        let localizer = LanguageLocalizer::new_with_language(&device.locale);
        let message = self.message(localizer, transaction.clone(), subscription.clone(), assets.clone())?;

        let notification_transaction = PushNotificationTransaction {
            wallet_index: subscription.wallet_index,
            transaction_id: transaction.id.to_string(),
            transaction: transaction.clone(),
            asset_id: transaction.asset_id.to_string(),
        };
        let data = PushNotification {
            notification_type: PushNotificationTypes::Transaction,
            data: serde_json::to_value(&notification_transaction).ok(),
        };

        let notification = GorushNotification::from_device(device, message.title, message.message.unwrap_or_default(), data);

        Ok(vec![notification])
    }
}
