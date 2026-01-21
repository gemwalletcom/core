use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use crate::devices::DeviceCacher;
use primitives::{Chain, WalletId, WalletSource, WalletSubscription, WalletSubscriptionChains};
use storage::models::NewWalletRow;
use storage::sql_types::WalletType;
use storage::{Database, WalletsRepository};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue};

#[derive(Clone)]
pub struct WalletsClient {
    database: Database,
    device_cacher: DeviceCacher,
    stream_producer: StreamProducer,
}

impl WalletsClient {
    pub fn new(database: Database, device_cacher: DeviceCacher, stream_producer: StreamProducer) -> Self {
        Self { database, device_cacher, stream_producer }
    }

    pub async fn get_subscriptions(&self, device_id: &str) -> Result<Vec<WalletSubscriptionChains>, Box<dyn Error + Send + Sync>> {
        let device_row_id = self.device_cacher.get_device_row_id(device_id).await?;
        let rows = self.database.wallets()?.get_subscriptions(device_row_id)?;

        Ok(rows
            .into_iter()
            .fold(BTreeMap::<String, (WalletId, Vec<Chain>)>::new(), |mut acc, (wallet_row, subscription_row, _address_row)| {
                let wallet_id = wallet_row.wallet_id.0.clone();
                acc.entry(wallet_id.id()).or_insert((wallet_id, Vec::new())).1.push(subscription_row.chain.0);
                acc
            })
            .into_values()
            .map(|(wallet_id, mut chains)| {
                chains.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
                WalletSubscriptionChains { wallet_id, chains }
            })
            .collect())
    }

    pub async fn add_subscriptions(&self, device_id: &str, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if wallet_subscriptions.is_empty() {
            return Ok(0);
        }

        let device_row_id = self.device_cacher.get_device_row_id(device_id).await?;
        let mut store = self.database.wallets()?;

        let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
        let mut wallet_ids: HashMap<String, i32> = store.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

        let new_wallets: Vec<NewWalletRow> = wallet_subscriptions
            .iter()
            .filter(|x| !wallet_ids.contains_key(&x.wallet_id.id()))
            .map(|x| NewWalletRow {
                identifier: x.wallet_id.id(),
                wallet_type: WalletType::from(x.wallet_id.wallet_type()),
                source: storage::sql_types::WalletSource::from(x.source.clone().unwrap_or(WalletSource::Import)),
            })
            .collect();

        if !new_wallets.is_empty() {
            let new_identifiers: Vec<String> = new_wallets.iter().map(|x| x.identifier.clone()).collect();
            store.create_wallets(new_wallets)?;
            wallet_ids.extend(store.get_wallets(new_identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)));
        }

        let subscriptions: Vec<(i32, Chain, String)> = wallet_subscriptions
            .iter()
            .filter_map(|ws| {
                wallet_ids.get(&ws.wallet_id.id()).map(|&wallet_id| ws.subscriptions.iter().map(move |s| (wallet_id, s.chain, s.address.clone())))
            })
            .flatten()
            .collect();

        let count = store.add_subscriptions(device_row_id, subscriptions)?;

        let payload: Vec<ChainAddressPayload> = wallet_subscriptions
            .into_iter()
            .filter(|x| x.source == Some(WalletSource::Import))
            .flat_map(|x| x.subscriptions)
            .map(ChainAddressPayload::from)
            .collect();

        if !payload.is_empty() {
            self.stream_producer.publish_new_addresses(payload).await?;
        }

        Ok(count)
    }

    pub async fn delete_subscriptions(&self, device_id: &str, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if wallet_subscriptions.is_empty() {
            return Ok(0);
        }

        let device_row_id = self.device_cacher.get_device_row_id(device_id).await?;
        let mut store = self.database.wallets()?;

        let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
        let wallet_ids: HashMap<String, i32> = store.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

        let subscriptions: Vec<(i32, Chain, String)> = wallet_subscriptions
            .into_iter()
            .filter_map(|ws| {
                wallet_ids.get(&ws.wallet_id.id()).map(|&wallet_id| ws.subscriptions.into_iter().map(move |s| (wallet_id, s.chain, s.address)))
            })
            .flatten()
            .collect();

        let count = store.delete_subscriptions(device_row_id, subscriptions)?;

        Ok(count)
    }
}
