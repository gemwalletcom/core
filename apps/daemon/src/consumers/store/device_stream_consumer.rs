use std::error::Error;

use async_trait::async_trait;
use cacher::CacherClient;
use primitives::{StreamBalanceUpdate, StreamEvent, StreamNftUpdate, StreamTransactionsUpdate, device_stream_channel};
use streamer::{DeviceStreamEvent, DeviceStreamPayload, consumer::MessageConsumer};

pub struct DeviceStreamConsumer {
    pub cacher_client: CacherClient,
}

#[async_trait]
impl MessageConsumer<DeviceStreamPayload, usize> for DeviceStreamConsumer {
    async fn should_process(&self, _payload: DeviceStreamPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: DeviceStreamPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let channel = device_stream_channel(&payload.device_id);

        match payload.event {
            DeviceStreamEvent::Transactions {
                wallet_id,
                transaction_ids,
                asset_ids,
            } => {
                let mut count = 0;

                if !asset_ids.is_empty() {
                    let balances: Vec<_> = asset_ids
                        .into_iter()
                        .map(|asset_id| StreamBalanceUpdate {
                            wallet_id: wallet_id.clone(),
                            asset_id,
                        })
                        .collect();
                    self.cacher_client.publish(&channel, &StreamEvent::Balances(balances)).await?;
                    count += 1;
                }

                if !transaction_ids.is_empty() {
                    let event = StreamEvent::Transactions(StreamTransactionsUpdate {
                        wallet_id,
                        transactions: transaction_ids,
                    });
                    self.cacher_client.publish(&channel, &event).await?;
                    count += 1;
                }

                Ok(count)
            }
            DeviceStreamEvent::Nft { wallet_id } => {
                self.cacher_client.publish(&channel, &StreamEvent::Nft(StreamNftUpdate { wallet_id })).await?;
                Ok(1)
            }
        }
    }
}
