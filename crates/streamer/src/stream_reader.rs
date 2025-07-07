use std::error::Error;

use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use serde::de::DeserializeOwned;

use crate::QueueName;

pub struct StreamReader {
    channel: Channel,
}

impl StreamReader {
    pub async fn new(url: &str, connection_name: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connection = Connection::connect(url, ConnectionProperties::default().with_connection_name(connection_name.into())).await?;
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
                &format!("consumer-{queue}"),
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
                            Ok(_) => self.ack(delivery_tag).await?,
                            Err(e) => {
                                self.nack(delivery_tag).await?;
                                return Err(e);
                            }
                        },
                        Err(e) => {
                            println!("Consumer deserialization error: {}, payload: {:?}", e, String::from_utf8_lossy(&delivery.data));
                            let _ = match self.nack(delivery_tag).await {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e),
                            };
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(())
    }

    async fn ack(&self, delivery_tag: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel
            .basic_ack(delivery_tag, BasicAckOptions { multiple: false })
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    async fn nack(&self, delivery_tag: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel
            .basic_nack(
                delivery_tag,
                BasicNackOptions {
                    multiple: false,
                    requeue: false,
                },
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }
}
