use std::collections::HashMap;
use std::error::Error;

use primitives::{Chain, ChainAddress, WalletIdType, WalletSource, WalletSubscription};
use storage::models::NewWalletRow;
use storage::sql_types::WalletType;
use storage::{Database, WalletsRepository};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue};

#[derive(Clone)]
pub struct WalletsClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl WalletsClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn get_subscriptions(&self, device_id: &str) -> Result<Vec<WalletSubscription>, Box<dyn Error + Send + Sync>> {
        let rows = self.database.wallets()?.get_subscriptions(device_id)?;

        let result = rows.into_iter().fold(
            HashMap::<String, (WalletIdType, WalletSource, Vec<ChainAddress>)>::new(),
            |mut acc, (wallet_row, subscription_row)| {
                let chain_address = ChainAddress::new(subscription_row.chain.0, subscription_row.address);
                acc.entry(wallet_row.identifier.clone())
                    .or_insert_with(|| {
                        let wallet_id =
                            WalletIdType::from_id(&wallet_row.identifier).unwrap_or(WalletIdType::Multicoin(wallet_row.identifier.clone()));
                        let source = wallet_row.source.0.clone();
                        (wallet_id, source, Vec::new())
                    })
                    .2
                    .push(chain_address);
                acc
            },
        );

        Ok(result
            .into_values()
            .map(|(wallet_id, source, subscriptions)| WalletSubscription { wallet_id, source, subscriptions })
            .collect())
    }

    pub async fn add_subscriptions(&self, device_id: &str, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut store = self.database.wallets()?;

        let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
        let mut wallet_ids: HashMap<String, i32> = store
            .get_wallets(identifiers)?
            .into_iter()
            .map(|x| (x.identifier, x.id))
            .collect();

        let new_wallets: Vec<NewWalletRow> = wallet_subscriptions
            .iter()
            .filter(|x| !wallet_ids.contains_key(&x.wallet_id.id()))
            .map(|x| NewWalletRow {
                identifier: x.wallet_id.id(),
                wallet_type: WalletType::from(x.wallet_id.wallet_type()),
                source: storage::sql_types::WalletSource::from(x.source.clone()),
            })
            .collect();

        if !new_wallets.is_empty() {
            let new_identifiers: Vec<String> = new_wallets.iter().map(|x| x.identifier.clone()).collect();
            store.create_wallets(new_wallets)?;
            wallet_ids.extend(store.get_wallets(new_identifiers)?.into_iter().map(|x| (x.identifier, x.id)));
        }

        let subscriptions: Vec<(String, Vec<(Chain, String)>)> = wallet_subscriptions
            .iter()
            .map(|x| {
                let addresses: Vec<(Chain, String)> = x.subscriptions.iter().map(|a| (a.chain, a.address.clone())).collect();
                (x.wallet_id.id(), addresses)
            })
            .collect();

        let count = store.add_subscriptions(device_id, wallet_ids, subscriptions)?;

        let payload: Vec<ChainAddressPayload> = wallet_subscriptions
            .into_iter()
            .filter(|x| x.source == WalletSource::Import)
            .flat_map(|x| x.subscriptions)
            .map(ChainAddressPayload::from)
            .collect();

        if !payload.is_empty() {
            self.stream_producer.publish_new_addresses(payload).await?;
        }

        Ok(count)
    }

    pub async fn delete_subscriptions(&self, device_id: &str, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut store = self.database.wallets()?;

        let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
        let wallet_ids: HashMap<String, i32> = store
            .get_wallets(identifiers)?
            .into_iter()
            .map(|x| (x.identifier, x.id))
            .collect();

        let subscriptions: Vec<(String, Vec<(Chain, String)>)> = wallet_subscriptions
            .into_iter()
            .map(|x| {
                let addresses: Vec<(Chain, String)> = x.subscriptions.into_iter().map(|a| (a.chain, a.address)).collect();
                (x.wallet_id.id(), addresses)
            })
            .collect();

        let count = store.delete_subscriptions(device_id, wallet_ids, subscriptions)?;

        Ok(count)
    }
}
