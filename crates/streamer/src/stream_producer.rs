use std::error::Error;

use lapin::{BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind, options::*, publisher_confirm::Confirmation, types::FieldTable};

use crate::{ExchangeName, QueueName, Retry, StreamConnection, with_retry};

const ROUTING_KEY_EXCHANGE_SUFFIX: &str = "_exchange";
const MAX_QUEUE_BYTES: i64 = 1_000_000_000;

#[derive(Clone)]
pub struct StreamProducerConfig {
    pub url: String,
    pub retry: Retry,
}

impl StreamProducerConfig {
    pub fn new(url: String, retry: Retry) -> Self {
        Self { url, retry }
    }
}

fn queue_args() -> FieldTable {
    let mut args = FieldTable::default();
    args.insert("x-max-length-bytes".into(), MAX_QUEUE_BYTES.into());
    args
}

#[derive(Clone)]
pub struct StreamProducer {
    channel: Channel,
}

impl StreamProducer {
    pub async fn new(config: &StreamProducerConfig, connection_name: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = with_retry(&config.retry, connection_name, || Self::try_connect(&config.url, connection_name)).await?;
        Ok(Self { channel })
    }

    pub async fn from_connection(connection: &StreamConnection) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = connection.create_channel().await?;
        Ok(Self { channel })
    }

    async fn try_connect(url: &str, name: &str) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let options = ConnectionProperties::default().with_connection_name(name.to_string().into());
        let connection = Connection::connect(url, options).await?;
        let channel = connection.create_channel().await?;
        Ok(channel)
    }

    // Queue methods

    pub async fn declare_queue(&self, name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel
            .queue_declare(
                name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                queue_args(),
            )
            .await?;
        Ok(())
    }

    pub async fn declare_queues(&self, queues: Vec<QueueName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for queue in queues {
            self.declare_queue(&queue.to_string()).await?;
        }
        Ok(())
    }

    pub async fn delete_queue(&self, queue: &str) -> Result<u32, Box<dyn Error + Send + Sync>> {
        Ok(self.channel.queue_delete(queue, QueueDeleteOptions::default()).await?)
    }

    pub async fn clear_queue(&self, queue: QueueName) -> Result<u32, Box<dyn Error + Send + Sync>> {
        Ok(self.channel.queue_purge(&queue.to_string(), QueuePurgeOptions::default()).await?)
    }

    // Exchange methods

    pub async fn declare_exchange(&self, name: &str, kind: ExchangeKind) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel.exchange_declare(name, kind, ExchangeDeclareOptions::default(), FieldTable::default()).await?;
        Ok(())
    }

    pub async fn declare_exchanges(&self, exchanges: Vec<ExchangeName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for exchange in exchanges {
            self.declare_exchange(&exchange.to_string(), exchange.kind()).await?;
        }
        Ok(())
    }

    // Bind methods

    pub async fn bind_queue(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.channel
            .queue_bind(queue, exchange, routing_key, QueueBindOptions::default(), FieldTable::default())
            .await?;
        Ok(())
    }

    pub async fn bind_exchange(&self, exchange: ExchangeName, queues: Vec<QueueName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for queue in queues {
            self.bind_queue(&queue.to_string(), &exchange.to_string(), "").await?;
        }
        Ok(())
    }

    pub async fn bind_queue_routing_key(&self, queue: QueueName, routing_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let exchange_name = format!("{}{}", queue, ROUTING_KEY_EXCHANGE_SUFFIX);
        let queue_name = format!("{}.{}", queue, routing_key);
        self.declare_queue(&queue_name).await?;
        self.bind_queue(&queue_name, &exchange_name, routing_key).await
    }

    // Publish methods

    async fn publish_message<T>(&self, exchange: &str, routing_key: &str, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let data = serde_json::to_vec(message)?;
        let confirm = self
            .channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &data,
                BasicProperties::default().with_delivery_mode(2).with_content_type("application/json".into()),
            )
            .await?;

        match confirm.await? {
            Confirmation::NotRequested | Confirmation::Ack(_) => Ok(true),
            Confirmation::Nack(_) => Ok(false),
        }
    }

    pub async fn publish<T>(&self, queue: QueueName, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        self.publish_message("", &queue.to_string(), message).await
    }

    pub async fn publish_batch<T>(&self, queue: QueueName, messages: &[T]) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        for message in messages {
            self.publish_message("", &queue.to_string(), message).await?;
        }
        Ok(true)
    }

    pub async fn publish_to_exchange<T>(&self, exchange: ExchangeName, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        self.publish_message(&exchange.to_string(), "", message).await
    }

    pub async fn publish_to_exchange_with_routing_key<T>(&self, exchange: ExchangeName, routing_key: &str, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        self.publish_message(&exchange.to_string(), routing_key, message).await
    }

    pub async fn publish_to_exchange_batch<T>(&self, exchange: ExchangeName, messages: &[T]) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        for message in messages {
            self.publish_message(&exchange.to_string(), "", message).await?;
        }
        Ok(true)
    }

    pub async fn publish_with_routing_key<T>(&self, queue: QueueName, routing_key: &str, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let exchange_name = format!("{}{}", queue, ROUTING_KEY_EXCHANGE_SUFFIX);
        self.publish_message(&exchange_name, routing_key, message).await
    }
}
