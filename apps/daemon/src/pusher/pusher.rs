use std::error::Error;

use localizer::LanguageLocalizer;
use number_formatter::{ValueFormatter, ValueStyle};
use primitives::{
    AddressFormatter, Asset, AssetVecExt, Chain, DeviceSubscription, GorushNotification, NFTAssetId, PushNotification, PushNotificationTransaction, PushNotificationTypes,
    Transaction, TransactionNFTTransferMetadata, TransactionSwapMetadata, TransactionType,
};
use storage::{Database, ScanAddressesRepository};

use api_connector::pusher::model::Message;

pub struct Pusher {
    database: Database,
}

impl Pusher {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_address(&self, chain: Chain, address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let result = self.database.scan_addresses()?.get_scan_address(chain, address);
        match result {
            Ok(address) => Ok(address.name.unwrap_or_default()),
            Err(_) => Ok(AddressFormatter::short(chain, address)),
        }
    }

    pub fn message(&self, localizer: LanguageLocalizer, transaction: Transaction, address: &str, assets: Vec<Asset>) -> Result<Message, Box<dyn Error + Send + Sync>> {
        let asset = assets.asset_result(transaction.asset_id.clone())?;
        let amount = ValueFormatter::format_with_symbol(ValueStyle::Auto, transaction.value.as_str(), asset.decimals, &asset.symbol)?;
        let chain = transaction.asset_id.chain;

        let to_address = self.get_address(chain, transaction.to.as_str())?;
        let from_address = self.get_address(chain, transaction.from.as_str())?;

        match transaction.transaction_type {
            TransactionType::Transfer | TransactionType::SmartContractCall => {
                let is_sent = transaction.is_sent(address.to_string());
                let title = localizer.notification_transfer_title(is_sent, &amount);
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
                let is_sent = transaction.is_sent(address.to_string());
                let title = localizer.notification_nft_transfer_title(is_sent, &name);
                let message = localizer.notification_transfer_description(is_sent, to_address.as_str(), from_address.as_str());
                Ok(Message { title, message: Some(message) })
            }
            TransactionType::TokenApproval => Ok(Message {
                title: localizer.notification_token_approval_title(asset.symbol.as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeDelegate => Ok(Message {
                title: localizer.notification_stake_title(&amount, to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeUndelegate => Ok(Message {
                title: localizer.notification_unstake_title(&amount, to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeRedelegate => Ok(Message {
                title: localizer.notification_redelegate_title(&amount, to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeRewards => Ok(Message {
                title: localizer.notification_claim_rewards_title(&amount),
                message: None,
            }),
            TransactionType::StakeWithdraw => Ok(Message {
                title: localizer.notification_withdraw_stake_title(&amount, to_address.as_str()),
                message: None,
            }),
            TransactionType::Swap => {
                let metadata = transaction.metadata.ok_or("Missing metadata")?;
                let metadata: TransactionSwapMetadata = serde_json::from_value(metadata)?;
                let from_asset = assets.asset_result(metadata.from_asset.clone())?;
                let to_asset = assets.asset_result(metadata.to_asset.clone())?;
                let from_amount = ValueFormatter::format_with_symbol(ValueStyle::Auto, &metadata.from_value, from_asset.decimals, &from_asset.symbol)?;
                let to_amount = ValueFormatter::format_with_symbol(ValueStyle::Auto, &metadata.to_value, to_asset.decimals, &to_asset.symbol)?;

                Ok(Message {
                    title: localizer.notification_swap_title(from_asset.symbol.as_str(), to_asset.symbol.as_str()),
                    message: Some(localizer.notification_swap_description(&from_amount, &to_amount)),
                })
            }
            TransactionType::PerpetualOpenPosition => {
                let title = format!("Opened Perpetual Position: {amount}");
                let message = format!("Opened perpetual position for {amount} at {to_address}");
                Ok(Message { title, message: Some(message) })
            }
            TransactionType::PerpetualClosePosition => {
                let title = format!("Closed Perpetual Position: {amount}");
                let message = format!("Closed perpetual position for {amount} at {to_address}");
                Ok(Message { title, message: Some(message) })
            }
            TransactionType::AssetActivation | TransactionType::PerpetualModifyPosition | TransactionType::EarnDeposit | TransactionType::EarnWithdraw => todo!(),
            TransactionType::StakeFreeze => Ok(Message {
                title: localizer.notification_freeze_title(&amount),
                message: None,
            }),
            TransactionType::StakeUnfreeze => Ok(Message {
                title: localizer.notification_unfreeze_title(&amount),
                message: None,
            }),
        }
    }

    pub async fn get_messages(
        &self,
        subscription: &DeviceSubscription,
        transaction: Transaction,
        assets: Vec<Asset>,
    ) -> Result<Vec<GorushNotification>, Box<dyn Error + Send + Sync>> {
        let transaction = transaction.finalize(vec![subscription.address.clone()]).without_utxo();

        let localizer = LanguageLocalizer::new_with_language(subscription.device.locale.as_str());
        let message = self.message(localizer, transaction.clone(), &subscription.address, assets.clone())?;

        let notification_transaction = PushNotificationTransaction {
            wallet_id: subscription.wallet_id.id(),
            transaction_id: transaction.id.to_string(),
            transaction: transaction.clone(),
            asset_id: transaction.asset_id.to_string(),
        };
        let data = PushNotification {
            notification_type: PushNotificationTypes::Transaction,
            data: serde_json::to_value(&notification_transaction).ok(),
        };

        Ok(
            GorushNotification::from_device(subscription.device.clone(), message.title, message.message.unwrap_or_default(), data)
                .into_iter()
                .collect(),
        )
    }
}
