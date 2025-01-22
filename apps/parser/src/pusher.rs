use std::error::Error;

use localizer::LanguageLocalizer;
use num_format::Locale;
use primitives::{
    AddressFormatter, BigNumberFormatter, Chain, PushNotification, PushNotificationTransaction, PushNotificationTypes, Subscription, Transaction,
    TransactionSwapMetadata, TransactionType,
};
use storage::models::Asset;
use storage::DatabaseClient;

use api_connector::pusher::model::Message;
use api_connector::PusherClient;

pub struct Pusher {
    client: PusherClient,
    database_client: DatabaseClient,
}

impl Pusher {
    pub fn new(database_url: String, pusher_client: PusherClient) -> Self {
        let database_client = DatabaseClient::new(&database_url);
        Self {
            client: pusher_client,
            database_client,
        }
    }

    pub fn format_amount(value: String, asset: &Asset, target_scale: i64) -> String {
        let scale = BigNumberFormatter::get_formatted_scale(value.as_str(), asset.decimals, target_scale).unwrap_or_default();
        BigNumberFormatter::formatted_value(value.as_str(), asset.decimals, scale, Some(Locale::en)).unwrap_or_default()
    }

    pub fn get_address(&mut self, chain: Chain, address: &str) -> Result<String, Box<dyn Error>> {
        let result = self.database_client.get_scan_address(chain, address);
        match result {
            Ok(address) => Ok(address.name.unwrap_or_default()),
            Err(_) => Ok(AddressFormatter::short(chain, address)),
        }
    }

    pub fn message(&mut self, localizer: LanguageLocalizer, transaction: Transaction, subscription: Subscription) -> Result<Message, Box<dyn Error>> {
        let asset = self.database_client.get_asset(transaction.asset_id.to_string().as_str())?;
        let amount = Pusher::format_amount(transaction.value.clone(), &asset, 2);
        let chain = transaction.asset_id.chain;
        let to_address = self.get_address(chain, transaction.to.as_str())?;
        let from_address = self.get_address(chain, transaction.from.as_str())?;

        match transaction.transaction_type {
            TransactionType::Transfer => {
                let is_sent = transaction.input_addresses().contains(&subscription.address) || transaction.from == subscription.address;

                let title = localizer.notification_transfer_title(is_sent, Pusher::get_value(amount, asset.symbol).as_str());

                let message = localizer.notification_transfer_description(is_sent, to_address.as_str(), from_address.as_str());

                Ok(Message { title, message: Some(message) })
            }
            TransactionType::TokenApproval => Ok(Message {
                title: localizer.notification_token_approval_title(asset.symbol.as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeDelegate => Ok(Message {
                title: localizer.notification_stake_title(Pusher::get_value(amount, asset.symbol).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeUndelegate => Ok(Message {
                title: localizer.notification_unstake_title(Pusher::get_value(amount, asset.symbol).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeRedelegate => Ok(Message {
                title: localizer.notification_redelegate_title(Pusher::get_value(amount, asset.symbol).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::StakeRewards => Ok(Message {
                title: localizer.notification_claim_rewards_title(Pusher::get_value(amount, asset.symbol).as_str()),
                message: None,
            }),
            TransactionType::StakeWithdraw => Ok(Message {
                title: localizer.notification_withdraw_stake_title(Pusher::get_value(amount, asset.symbol).as_str(), to_address.as_str()),
                message: None,
            }),
            TransactionType::Swap => {
                let metadata = transaction.metadata.ok_or("Missing metadata")?;
                let metadata: TransactionSwapMetadata = serde_json::from_value(metadata)?;
                let from_asset = self.database_client.get_asset(metadata.from_asset.to_string().as_str())?;
                let to_asset = self.database_client.get_asset(metadata.to_asset.to_string().as_str())?;
                let from_amount = BigNumberFormatter::value(metadata.from_value.as_str(), from_asset.decimals).unwrap_or_default();
                let to_amount = BigNumberFormatter::value(metadata.to_value.as_str(), to_asset.decimals).unwrap_or_default();

                Ok(Message {
                    title: localizer.notification_swap_title(from_asset.symbol.as_str(), to_asset.symbol.as_str()),
                    message: Some(localizer.notification_swap_description(
                        Pusher::get_value(from_amount, from_asset.symbol).as_str(),
                        Pusher::get_value(to_amount, to_asset.symbol).as_str(),
                    )),
                })
            }
            TransactionType::AssetActivation => todo!(),
        }
    }

    pub fn get_value(amount: String, symbol: String) -> String {
        format! {"{} {}", amount, symbol}
    }

    pub async fn push(&mut self, device: primitives::Device, transaction: Transaction, subscription: Subscription) -> Result<usize, Box<dyn Error>> {
        // only push if push is enabled and token is set
        if !device.is_push_enabled || device.token.is_empty() {
            return Ok(0);
        }
        let localizer = LanguageLocalizer::new_with_language(&device.locale);
        let message = self.message(localizer, transaction.clone(), subscription.clone())?;

        let notification_transaction = PushNotificationTransaction {
            wallet_index: subscription.wallet_index,
            transaction_id: transaction.id,
            asset_id: transaction.asset_id.to_string(),
        };
        let data = PushNotification {
            notification_type: PushNotificationTypes::Transaction,
            data: serde_json::to_value(&notification_transaction).ok(),
        };

        let notification = self.client.new_notification(
            device.token.as_str(),
            device.platform,
            message.title.as_str(),
            message.message.unwrap_or_default().as_str(),
            data,
        );

        let response = self.client.push(notification).await?;

        if !response.logs.is_empty() {
            println!("push logs: {:?}", response.logs);
            let _ = self.database_client.update_device_is_push_enabled(&device.id, false)?;
        }

        Ok(response.counts as usize)
    }
}

#[cfg(test)]
mod tests {

    use primitives::{asset_constants::USDT_ETH_ASSET_ID, AssetType};

    use super::*;

    #[test]
    fn test_usdt_format_amount() {
        let asset = Asset::from_primitive(primitives::Asset {
            id: primitives::AssetId::from_token(Chain::Ethereum, USDT_ETH_ASSET_ID),
            name: "USDT".to_string(),
            symbol: "USDT".to_string(),
            decimals: 6,
            asset_type: AssetType::ERC20,
        });
        let amount = "1123450000".to_string();
        let formatted_amount = Pusher::format_amount(amount.clone(), &asset, 2);

        assert_eq!(formatted_amount, "1,123.45");
        assert_eq!(Pusher::get_value(formatted_amount, asset.symbol), "1,123.45 USDT");
    }

    #[test]
    fn test_long_usdt_format_amount() {
        let asset = Asset::from_primitive(primitives::Asset {
            id: primitives::AssetId::from_token(Chain::Ethereum, USDT_ETH_ASSET_ID),
            name: "USDT".to_string(),
            symbol: "USDT".to_string(),
            decimals: 6,
            asset_type: AssetType::ERC20,
        });

        let amount = "1123459999".to_string();
        let formatted_amount = Pusher::format_amount(amount.clone(), &asset, 2);

        assert_eq!(formatted_amount, "1,123.46");
        assert_eq!(Pusher::get_value(formatted_amount, asset.symbol), "1,123.46 USDT");
    }

    #[test]
    fn test_btc_format_amount() {
        let asset = Asset::from_primitive(primitives::Asset::from_chain(Chain::Bitcoin));
        let amount = "12000".to_string();

        let formatted_amount = Pusher::format_amount(amount.clone(), &asset, 2);
        assert_eq!(formatted_amount, "0.00012");
        assert_eq!(Pusher::get_value(formatted_amount, asset.symbol), "0.00012 BTC");
    }
}
