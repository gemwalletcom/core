use std::error::Error;

use futures::StreamExt;
use gem_tracing::{error_fields, error_with_fields, info_with_fields};
use lapin::{Channel, Connection, ConnectionProperties, options::*, types::FieldTable};
use serde::de::DeserializeOwned;

use crate::{QueueName, Retry, ShutdownReceiver, StreamConnection, with_retry};

#[derive(Clone)]
pub struct StreamReaderConfig {
    pub url: String,
    pub name: String,
    pub prefetch: u16,
    pub retry: Retry,
}

impl StreamReaderConfig {
    pub fn new(url: String, name: String, prefetch: u16, retry: Retry) -> Self {
        Self { url, name, prefetch, retry }
    }
}

pub struct StreamReader {
    config: StreamReaderConfig,
    channel: Channel,
}

impl StreamReader {
    pub async fn new(config: StreamReaderConfig, shutdown_rx: &ShutdownReceiver) -> Result<Option<Self>, Box<dyn Error + Send + Sync>> {
        let channel = with_retry(&config.retry, &config.name, shutdown_rx, || Self::try_connect(&config)).await?;
        Ok(channel.map(|channel| Self { config, channel }))
    }

    pub async fn from_connection(connection: &StreamConnection, config: StreamReaderConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config = StreamReaderConfig {
            url: connection.url().to_string(),
            name: connection.name().to_string(),
            ..config
        };
        let channel = Self::create_channel(connection, config.prefetch).await?;
        Ok(Self { config, channel })
    }

    async fn try_connect(config: &StreamReaderConfig) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let options = ConnectionProperties::default().with_connection_name(config.name.clone().into());
        let connection = Connection::connect(&config.url, options).await?;
        Self::configure_channel(connection.create_channel().await?, config.prefetch).await
    }

    pub async fn close(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel.close(0, "Normal shutdown".into()).await?;
        Ok(())
    }

    async fn create_channel(connection: &StreamConnection, prefetch: u16) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        Self::configure_channel(connection.create_channel().await?, prefetch).await
    }

    async fn configure_channel(channel: Channel, prefetch: u16) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        channel.basic_qos(prefetch, BasicQosOptions { global: false }).await?;
        Ok(channel)
    }

    async fn reconnect(&mut self, shutdown_rx: &ShutdownReceiver) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let result = with_retry(&self.config.retry, &self.config.name, shutdown_rx, || Self::try_connect(&self.config)).await?;

        match result {
            Some(channel) => {
                self.channel = channel;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn read<T, F>(&mut self, queue: QueueName, routing_key: Option<&str>, mut callback: F, shutdown_rx: ShutdownReceiver) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Result<(), Box<dyn Error + Send + Sync>>,
    {
        let (queue_name, consumer_tag) = match routing_key {
            Some(key) => (format!("{}.{}", queue, key), format!("consumer-{}-{}", queue, key)),
            None => (queue.to_string(), format!("consumer-{queue}")),
        };

        loop {
            if *shutdown_rx.borrow() {
                break;
            }

            let consumer_result = self
                .channel
                .basic_consume(
                    queue_name.as_str().into(),
                    consumer_tag.as_str().into(),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await;

            let mut consumer = match consumer_result {
                Ok(c) => c,
                Err(e) => {
                    error_fields!("consumer setup failed", connection = self.config.name.as_str(), error = format!("{e}"));
                    if !self.reconnect(&shutdown_rx).await? {
                        break;
                    }
                    continue;
                }
            };

            let result = self.consume::<T, _>(&mut consumer, &mut callback, shutdown_rx.clone()).await;
            if let Ok(true) = result {
                break;
            }
            let error = result.err().map(|e| e.to_string());
            info_with_fields!(
                "consumer reconnecting",
                connection = self.config.name.as_str(),
                error = error.as_deref().unwrap_or("stream ended")
            );
            if !self.reconnect(&shutdown_rx).await? {
                break;
            }
        }

        Ok(())
    }

    async fn consume<T, F>(&mut self, consumer: &mut lapin::Consumer, callback: &mut F, mut shutdown_rx: ShutdownReceiver) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Result<(), Box<dyn Error + Send + Sync>>,
    {
        loop {
            let delivery = tokio::select! {
                d = consumer.next() => d,
                _ = shutdown_rx.changed() => return Ok(true),
            };

            match delivery {
                Some(Ok(delivery)) => {
                    let delivery_tag = delivery.delivery_tag;
                    let data = serde_json::from_slice::<T>(&delivery.data);
                    match data {
                        Ok(obj) => match callback(obj) {
                            Ok(_) => self.ack(delivery_tag).await?,
                            Err(_) => self.nack(delivery_tag, true).await?,
                        },
                        Err(e) => {
                            error_with_fields!("deserialization error", &e, payload = String::from_utf8_lossy(&delivery.data).to_string());
                            let _ = self.nack(delivery_tag, false).await;
                        }
                    }
                }
                Some(Err(e)) => return Err(Box::new(e)),
                None => return Ok(false),
            }
        }
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
