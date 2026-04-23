use std::collections::HashSet;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::{AssetAddress, DeviceSubscription, Transaction, TransactionId, TransactionState, TransactionType};
use storage::{AssetsAddressesRepository, AssetsRepository, Database, TransactionsRepository, WalletsRepository};
use streamer::{AssetId, NotificationsPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, WalletStreamEvent, WalletStreamPayload, consumer::MessageConsumer};
use swapper::cross_chain::{self, DepositAddressMap, SendAddressMap};

use crate::client::SwapVaultAddressClient;
use crate::consumers::store::StoreTransactionsConsumerConfig;
use crate::pusher::Pusher;

const TRANSACTION_BATCH_SIZE: usize = 100;

const IN_TRANSIT_TYPES: [TransactionType; 2] = [TransactionType::Transfer, TransactionType::SmartContractCall];

pub struct StoreTransactionsConsumer {
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub pusher: Pusher,
    pub config: StoreTransactionsConsumerConfig,
    pub vault_client: SwapVaultAddressClient,
}

struct ProcessingResult {
    transactions: Vec<Transaction>,
    assets_addresses: Vec<AssetAddress>,
    notifications: Vec<NotificationsPayload>,
    wallet_events: Vec<WalletStreamPayload>,
}

#[async_trait]
impl MessageConsumer<TransactionsPayload, usize> for StoreTransactionsConsumer {
    async fn should_process(&self, _payload: TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.chain;
        let is_notify_devices = payload.should_notify_devices();
        let deposit_addresses = self.vault_client.get_deposit_address_map().await?;
        let send_addresses = self.vault_client.get_send_address_map().await?;
        let transactions = Self::transactions_for_storage(payload.transactions, &deposit_addresses, &send_addresses);

        let min_amount = self.config.min_amount_usd;

        let addresses: Vec<_> = transactions.iter().flat_map(|tx| tx.addresses()).collect::<HashSet<_>>().into_iter().collect();
        let subscriptions = self.database.wallets()?.get_subscriptions_by_chain_addresses(chain, addresses)?;
        let notification_subscriptions = Self::unique_subscriptions_per_device(subscriptions.clone());

        let subscription_addresses: HashSet<_> = subscriptions.iter().map(|s| &s.address).collect();

        let asset_ids: Vec<AssetId> = transactions
            .iter()
            .filter(|x| x.addresses().iter().any(|addr| subscription_addresses.contains(addr)))
            .flat_map(|x| x.asset_ids())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let (existing_assets, missing_assets) = self.get_existing_and_missing_assets(asset_ids).await?;
        let existing_assets_map: HashMap<AssetId, primitives::AssetPriceMetadata> = existing_assets.into_iter().map(|asset| (asset.asset.asset.id.clone(), asset)).collect();

        let _ = self.stream_producer.publish_fetch_assets(missing_assets).await;

        let mut transactions_map: HashMap<TransactionId, Transaction> = HashMap::new();
        let mut assets_addresses = HashSet::new();
        let mut notifications: Vec<NotificationsPayload> = Vec::new();
        let mut wallet_events_map: HashMap<i32, (HashSet<TransactionId>, HashSet<AssetId>)> = HashMap::new();

        for subscription in &subscriptions {
            for transaction in &transactions {
                if !transaction.addresses().contains(&subscription.address) {
                    continue;
                }

                let transaction_asset_ids = transaction.asset_ids();

                if transaction_asset_ids.iter().any(|id| !existing_assets_map.contains_key(id)) {
                    continue;
                }

                let Some(asset_price) = existing_assets_map.get(&transaction.asset_id) else {
                    continue;
                };

                if self
                    .config
                    .is_transaction_insufficient_amount(transaction, &asset_price.asset.asset, asset_price.price, min_amount)
                {
                    continue;
                }

                if Self::should_store_asset_addresses(transaction) {
                    assets_addresses.extend(
                        transaction
                            .assets_addresses_with_fee()
                            .into_iter()
                            .filter(|address| address.address == subscription.address)
                            .filter(|address| existing_assets_map.contains_key(&address.asset_id)),
                    );
                }

                transactions_map.entry(transaction.id.clone()).or_insert_with(|| transaction.clone());

                let (txn_ids, asset_ids) = wallet_events_map.entry(subscription.wallet_row_id).or_default();
                txn_ids.insert(transaction.id.clone());
                asset_ids.extend(transaction_asset_ids.iter().cloned());
            }
        }

        for subscription in &notification_subscriptions {
            for transaction in transactions_map.values() {
                if !transaction.addresses().contains(&subscription.address) {
                    continue;
                }

                if !self.config.should_notify_transaction(transaction, is_notify_devices, &send_addresses) {
                    continue;
                }

                let transaction_asset_ids = transaction.asset_ids();
                let assets: Vec<primitives::Asset> = transaction_asset_ids
                    .iter()
                    .filter_map(|id| existing_assets_map.get(id))
                    .map(|asset_price| asset_price.asset.asset.clone())
                    .collect();

                if self.database.transactions()?.get_transaction_exists(&transaction.id)? {
                    continue;
                }

                if let Ok(push_notifications) = self.pusher.get_messages(subscription, transaction.clone(), assets).await {
                    notifications.push(NotificationsPayload::new(push_notifications));
                }
            }
        }

        let wallet_events = wallet_events_map
            .into_iter()
            .map(|(wallet_id, (transaction_ids, asset_ids))| WalletStreamPayload {
                wallet_id,
                event: WalletStreamEvent::Transactions {
                    transaction_ids: transaction_ids.into_iter().collect(),
                    asset_ids: asset_ids.into_iter().collect(),
                },
            })
            .collect();

        let transactions: Vec<_> = transactions_map.into_values().collect();
        let result = ProcessingResult {
            transactions,
            assets_addresses: assets_addresses.into_iter().collect(),
            notifications,
            wallet_events,
        };
        self.publish_results(result).await
    }
}

impl StoreTransactionsConsumer {
    fn should_store_asset_addresses(transaction: &Transaction) -> bool {
        match transaction.state {
            TransactionState::Confirmed | TransactionState::InTransit => true,
            TransactionState::Pending | TransactionState::Failed | TransactionState::Reverted => false,
        }
    }

    fn unique_subscriptions_per_device(subscriptions: Vec<DeviceSubscription>) -> Vec<DeviceSubscription> {
        subscriptions
            .into_iter()
            .fold(HashMap::<(String, String), DeviceSubscription>::new(), |mut best, sub| {
                let key = (sub.device.id.clone(), sub.address.clone());
                best.entry(key)
                    .and_modify(|existing| {
                        if sub.wallet_id.wallet_type().notification_priority() < existing.wallet_id.wallet_type().notification_priority() {
                            *existing = sub.clone();
                        }
                    })
                    .or_insert(sub);
                best
            })
            .into_values()
            .collect()
    }

    fn transactions_for_storage(transactions: Vec<Transaction>, deposit_addresses: &DepositAddressMap, send_addresses: &SendAddressMap) -> Vec<Transaction> {
        transactions
            .into_iter()
            .filter_map(|mut transaction| {
                if cross_chain::is_from_vault_address(&transaction, send_addresses) {
                    return None;
                }

                if transaction.state == TransactionState::Confirmed
                    && IN_TRANSIT_TYPES.contains(&transaction.transaction_type)
                    && cross_chain::is_cross_chain_swap(&transaction, deposit_addresses)
                {
                    transaction.state = TransactionState::InTransit;
                }

                Some(transaction)
            })
            .collect()
    }

    async fn publish_results(&self, result: ProcessingResult) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions_count = self.store_transactions(result.transactions).await?;
        self.database.assets_addresses()?.add_assets_addresses(result.assets_addresses)?;
        let _ = self.stream_producer.publish_notifications_transactions(result.notifications).await;
        let _ = self.stream_producer.publish_wallet_stream_events(result.wallet_events).await;

        Ok(transactions_count)
    }

    async fn get_existing_and_missing_assets(&self, assets_ids: Vec<AssetId>) -> Result<(Vec<primitives::AssetPriceMetadata>, Vec<AssetId>), Box<dyn Error + Send + Sync>> {
        let assets_with_prices = self.database.assets()?.get_assets_with_prices(assets_ids.clone(), self.config.primary_price_max_age)?;

        let missing_assets_ids = assets_ids
            .into_iter()
            .filter(|asset_id| !assets_with_prices.iter().any(|a| &a.asset.asset.id == asset_id))
            .collect::<Vec<_>>();

        Ok((assets_with_prices, missing_assets_ids))
    }

    async fn store_transactions(&self, transactions: Vec<Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if transactions.is_empty() {
            return Ok(0);
        }

        for chunk in transactions.chunks(TRANSACTION_BATCH_SIZE) {
            self.database.transactions()?.add_transactions(chunk.to_vec())?;
        }

        Ok(transactions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, Device, SwapProvider, WalletId};

    #[test]
    fn test_transactions_for_storage() {
        let thorchain_vault = "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string();
        let near_vault = "TMoD2uJiUAvB2RhLGm1BmzCVVzi5VLFDVt".to_string();
        let deposit_addresses = DepositAddressMap::from([(thorchain_vault.clone(), SwapProvider::Thorchain), (near_vault.clone(), SwapProvider::NearIntents)]);
        let send_addresses = SendAddressMap::from([(thorchain_vault.clone(), SwapProvider::Thorchain), (near_vault.clone(), SwapProvider::NearIntents)]);

        let cross_chain = Transaction {
            to: thorchain_vault.clone(),
            memo: Some("=:BTC:bc1qaddress:0/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![cross_chain], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::InTransit
        );

        let vault_no_memo = Transaction {
            to: "bc1qvault".to_string(),
            ..Transaction::mock()
        };
        let deposit_addresses_bc = DepositAddressMap::from([("bc1qvault".to_string(), SwapProvider::Thorchain)]);
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![vault_no_memo], &deposit_addresses_bc, &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![Transaction::mock()], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let swap_type = Transaction {
            transaction_type: TransactionType::Swap,
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![swap_type], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let token_approval = Transaction {
            transaction_type: TransactionType::TokenApproval,
            to: "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2".to_string(),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![token_approval], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let pending = Transaction {
            state: TransactionState::Pending,
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![pending], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Pending
        );

        let near_intents = Transaction {
            to: near_vault.clone(),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![near_intents], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::InTransit
        );

        let outbound = Transaction {
            from: thorchain_vault.clone(),
            ..Transaction::mock()
        };
        let regular = Transaction::mock();

        let transactions = StoreTransactionsConsumer::transactions_for_storage(vec![outbound, regular.clone()], &DepositAddressMap::new(), &send_addresses);

        assert_eq!(transactions, vec![regular]);
    }

    #[test]
    fn test_should_store_asset_addresses() {
        assert!(StoreTransactionsConsumer::should_store_asset_addresses(&Transaction::mock()));
        assert!(StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::InTransit,
            ..Transaction::mock()
        }));
        assert!(!StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        }));
        assert!(!StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::Failed,
            ..Transaction::mock()
        }));
        assert!(!StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::Reverted,
            ..Transaction::mock()
        }));
    }

    #[test]
    fn test_unique_subscriptions_per_device() {
        let multicoin = DeviceSubscription::mock();
        let single = DeviceSubscription {
            wallet_id: WalletId::Single(Chain::Ethereum, "0xABC".to_string()),
            ..DeviceSubscription::mock()
        };
        let view = DeviceSubscription {
            wallet_id: WalletId::View(Chain::Ethereum, "0xABC".to_string()),
            ..DeviceSubscription::mock()
        };

        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![view.clone(), single.clone(), multicoin.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet_id, multicoin.wallet_id);

        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![view.clone(), single.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet_id, single.wallet_id);

        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![view.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet_id, view.wallet_id);

        let other_device = DeviceSubscription {
            device: Device {
                id: "device-2".to_string(),
                ..Device::mock()
            },
            wallet_id: WalletId::View(Chain::Ethereum, "0xABC".to_string()),
            ..DeviceSubscription::mock()
        };
        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![multicoin.clone(), other_device.clone()]);
        assert_eq!(result.len(), 2);
    }
}
