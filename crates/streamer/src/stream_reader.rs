use std::error::Error;

use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use serde::de::DeserializeOwned;

use crate::QueueName;

pub struct StreamReader {
    channel: Channel,
}

impl StreamReader {
    pub async fn new(url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        Ok(Self { channel })
    }

    pub async fn close(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel.close(0, "Normal shutdown").await?;
        Ok(())
    }

    pub async fn read<T, F>(&mut self, queue: QueueName, mut callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Result<(), Box<dyn Error + Send + Sync>>,
    {
        let mut consumer = self
            .channel
            .basic_consume(
                &queue.to_string(),
                &format!("consumer-{}", queue),
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let delivery_tag = delivery.delivery_tag;
                    let data = serde_json::from_slice::<T>(&delivery.data);
                    match data {
                        Ok(obj) => match callback(obj) {
                            Ok(_) => self.channel.basic_ack(delivery_tag, BasicAckOptions { multiple: false }).await?,
                            Err(e) => {
                                self.channel.basic_reject(delivery_tag, BasicRejectOptions { requeue: true }).await?;
                                return Err(e);
                            }
                        },
                        Err(e) => {
                            self.channel.basic_reject(delivery_tag, BasicRejectOptions { requeue: true }).await?;
                            return Err(Box::new(e));
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(())
    }
}
