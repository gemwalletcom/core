use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use gem_tracing::info_with_fields;
use lapin::{BasicProperties, Channel, Confirmation, Connection, ConnectionProperties, ExchangeKind, options::*, types::FieldTable};
use tokio::sync::Mutex;

use crate::{ExchangeName, QueueName, Retry, ShutdownReceiver, StreamConnection, with_retry};

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
    url: String,
    connection_name: String,
    retry: Retry,
    shutdown_rx: ShutdownReceiver,
    channel: Arc<Mutex<Channel>>,
}

impl StreamProducer {
    pub async fn new(config: &StreamProducerConfig, connection_name: &str, shutdown_rx: ShutdownReceiver) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = with_retry(&config.retry, connection_name, &shutdown_rx, || Self::try_connect(&config.url, connection_name))
            .await?
            .ok_or("shutdown during connect")?;
        Ok(Self {
            url: config.url.clone(),
            connection_name: connection_name.to_string(),
            retry: config.retry.clone(),
            shutdown_rx,
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    pub async fn from_connection(connection: &StreamConnection, shutdown_rx: ShutdownReceiver) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = connection.create_channel().await?;
        let retry = Retry::new(std::time::Duration::from_secs(1), std::time::Duration::from_secs(30));
        Ok(Self {
            url: connection.url().to_string(),
            connection_name: connection.name().to_string(),
            retry,
            shutdown_rx,
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    async fn try_connect(url: &str, name: &str) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let options = ConnectionProperties::default().with_connection_name(name.to_string().into());
        let connection = Connection::connect(url, options).await?;
        let channel = connection.create_channel().await?;
        Ok(channel)
    }

    async fn reconnect(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let mut guard = self.channel.lock().await;
        let channel = with_retry(&self.retry, &self.connection_name, &self.shutdown_rx, || {
            Self::try_connect(&self.url, &self.connection_name)
        })
        .await?
        .ok_or("shutdown during reconnect")?;
        *guard = channel;
        Ok(guard.clone())
    }

    async fn channel(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let channel = self.channel.lock().await.clone();
        if channel.status().connected() {
            return Ok(channel);
        }
        self.reconnect().await
    }

    async fn run<T, F, Fut>(&self, mut operation: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnMut(Channel) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let mut delay = self.retry.delay;
        let mut attempt = 0;

        loop {
            if *self.shutdown_rx.borrow() {
                return Err("shutdown during operation".into());
            }

            let channel = self.channel().await?;
            match operation(channel).await {
                Ok(value) => return Ok(value),
                Err(error) => {
                    attempt += 1;
                    info_with_fields!(
                        "rabbitmq producer retry",
                        connection = self.connection_name.as_str(),
                        attempt = attempt,
                        delay_secs = delay.as_secs(),
                        error = error.to_string()
                    );
                    let _ = self.reconnect().await;
                    let mut shutdown_rx = self.shutdown_rx.clone();
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {}
                        _ = shutdown_rx.changed() => return Err("shutdown during operation".into()),
                    }
                    delay = next_delay(delay, &self.retry);
                }
            }
        }
    }

    // Queue methods

    pub async fn declare_queue(&self, name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.run(|channel| async move {
            channel
                .queue_declare(
                    name.into(),
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    queue_args(),
                )
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn declare_queues(&self, queues: Vec<QueueName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for queue in queues {
            self.declare_queue(&queue.to_string()).await?;
        }
        Ok(())
    }

    pub async fn delete_queue(&self, queue: &str) -> Result<u32, Box<dyn Error + Send + Sync>> {
        self.run(|channel| async move { Ok(channel.queue_delete(queue.into(), QueueDeleteOptions::default()).await?) })
            .await
    }

    pub async fn clear_queue(&self, queue: QueueName) -> Result<u32, Box<dyn Error + Send + Sync>> {
        let queue_name = queue.to_string();
        self.run(|channel| {
            let queue_name = queue_name.clone();
            async move { Ok(channel.queue_purge(queue_name.as_str().into(), QueuePurgeOptions::default()).await?) }
        })
        .await
    }

    // Exchange methods

    pub async fn declare_exchange(&self, name: &str, kind: ExchangeKind) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.run(|channel| {
            let kind = kind.clone();
            async move {
                channel
                    .exchange_declare(name.into(), kind, ExchangeDeclareOptions::default(), FieldTable::default())
                    .await?;
                Ok(())
            }
        })
        .await
    }

    pub async fn declare_exchanges(&self, exchanges: Vec<ExchangeName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for exchange in exchanges {
            self.declare_exchange(&exchange.to_string(), exchange.kind()).await?;
        }
        Ok(())
    }

    // Bind methods

    pub async fn bind_queue(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.run(|channel| async move {
            channel
                .queue_bind(queue.into(), exchange.into(), routing_key.into(), QueueBindOptions::default(), FieldTable::default())
                .await?;
            Ok(())
        })
        .await
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
        self.run(|channel| {
            let data = data.clone();
            async move {
                let confirm = channel
                    .basic_publish(
                        exchange.into(),
                        routing_key.into(),
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
        })
        .await
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
        let queue_name = queue.to_string();
        for message in messages {
            if !self.publish_message("", &queue_name, message).await? {
                return Ok(false);
            }
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
        let exchange_name = exchange.to_string();
        for message in messages {
            if !self.publish_message(&exchange_name, "", message).await? {
                return Ok(false);
            }
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

fn next_delay(delay: Duration, retry: &Retry) -> Duration {
    (delay * 2).min(retry.timeout)
}
