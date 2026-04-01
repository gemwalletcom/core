use std::error::Error;

use async_trait::async_trait;
use cacher::CacherClient;
use primitives::{StreamBalanceUpdate, StreamEvent, StreamTransactionsUpdate, StreamWalletUpdate, device_stream_channel};
use storage::{Database, WalletsRepository};
use streamer::{WalletStreamEvent, WalletStreamPayload, consumer::MessageConsumer};

pub struct WalletStreamConsumer {
    pub database: Database,
    pub cacher_client: CacherClient,
}

#[async_trait]
impl MessageConsumer<WalletStreamPayload, usize> for WalletStreamConsumer {
    async fn should_process(&self, _payload: WalletStreamPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: WalletStreamPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let wallet = self.database.wallets()?.get_wallet_by_id(payload.wallet_id)?;
        let devices = self.database.wallets()?.get_devices_by_wallet_id(payload.wallet_id)?;
        let wallet_id = wallet.wallet_id.0;

        let events: Vec<StreamEvent> = match payload.event {
            WalletStreamEvent::Transactions { transaction_ids, asset_ids } => {
                let balances = asset_ids.into_iter().map(|asset_id| StreamBalanceUpdate { wallet_id: wallet_id.clone(), asset_id }).collect();
                vec![
                    StreamEvent::Balances(balances),
                    StreamEvent::Transactions(StreamTransactionsUpdate { wallet_id, transactions: transaction_ids }),
                ]
            }
            WalletStreamEvent::FiatTransaction => vec![StreamEvent::FiatTransaction(StreamWalletUpdate { wallet_id })],
            WalletStreamEvent::Nft => vec![StreamEvent::Nft(StreamWalletUpdate { wallet_id })],
            WalletStreamEvent::Perpetual => vec![StreamEvent::Perpetual(StreamWalletUpdate { wallet_id })],
        };

        let mut count = 0;
        for device in &devices {
            let channel = device_stream_channel(&device.device_id);
            for event in &events {
                self.cacher_client.publish(&channel, event).await?;
                count += 1;
            }
        }
        Ok(count)
    }
}
