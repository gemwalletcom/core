use std::error::Error;

use futures::StreamExt;
use lapin::{Channel, Connection, ConnectionProperties, options::*, types::FieldTable};
use serde::de::DeserializeOwned;

use crate::{QueueName, StreamConnection};

pub struct StreamReaderConfig {
    pub url: String,
    pub name: String,
    pub prefetch: u16,
}

impl StreamReaderConfig {
    pub fn new(url: String, name: String, prefetch: u16) -> Self {
        Self { url, name, prefetch }
    }
}

pub struct StreamReader {
    channel: Channel,
}

impl StreamReader {
    pub async fn new(config: StreamReaderConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connection = Connection::connect(&config.url, ConnectionProperties::default().with_connection_name(config.name.into())).await?;
        let channel = connection.create_channel().await?;
        channel.basic_qos(config.prefetch, BasicQosOptions { global: false }).await?;
        Ok(Self { channel })
    }

    pub async fn from_connection(connection: &StreamConnection, prefetch: u16) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = connection.create_channel().await?;
        channel.basic_qos(prefetch, BasicQosOptions { global: false }).await?;
        Ok(Self { channel })
    }

    pub async fn close(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel.close(0, "Normal shutdown").await?;
        Ok(())
    }

    pub async fn read<T, F>(&mut self, queue: QueueName, routing_key: Option<&str>, callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Result<(), Box<dyn Error + Send + Sync>>,
    {
        let (queue_name, consumer_tag) = match routing_key {
            Some(key) => (format!("{}.{}", queue, key), format!("consumer-{}-{}", queue, key)),
            None => (queue.to_string(), format!("consumer-{queue}")),
        };
        self.consume(&queue_name, &consumer_tag, callback).await
    }

    async fn consume<T, F>(&mut self, queue_name: &str, consumer_tag: &str, mut callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Result<(), Box<dyn Error + Send + Sync>>,
    {
        let mut consumer = self
            .channel
            .basic_consume(
                queue_name,
                consumer_tag,
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
                            Err(_) => self.nack(delivery_tag, true).await?,
                        },
                        Err(e) => {
                            println!("Consumer deserialization error: {}, payload: {:?}", e, String::from_utf8_lossy(&delivery.data));
                            let _ = self.nack(delivery_tag, false).await;
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

    async fn nack(&self, delivery_tag: u64, requeue: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel
            .basic_nack(delivery_tag, BasicNackOptions { multiple: false, requeue })
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }
}
