use std::error::Error;

use lapin::{options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind};

use crate::{ExchangeName, QueueName};

pub struct StreamProducer {
    channel: Channel,
}

impl StreamProducer {
    pub async fn new(url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;
        Ok(Self { channel })
    }

    pub async fn declare_queue(&self, queue: QueueName) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut args = FieldTable::default();
        args.insert("x-max-length-bytes".into(), 1_000_000_000i64.into());

        self.channel
            .queue_declare(
                &queue.to_string(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                args,
            )
            .await?;

        Ok(())
    }

    pub async fn declare_exchange(&self, exchange: ExchangeName) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(self
            .channel
            .exchange_declare(
                &exchange.to_string(),
                ExchangeKind::Fanout,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?)
    }

    pub async fn bind_exchange(&self, exchange: ExchangeName, queues: Vec<QueueName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for queue in queues {
            self.channel
                .queue_bind(
                    &queue.to_string(),
                    &exchange.to_string(),
                    "",
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }
        Ok(())
    }

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

        let confirmation = confirm.await?;

        match confirmation {
            Confirmation::NotRequested => Ok(true),
            Confirmation::Ack(_) => Ok(true),
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
        if messages.is_empty() {
            return Ok(true);
        }
        let queue_str = queue.to_string();
        for message in messages {
            self.publish_message("", &queue_str, message).await?;
        }
        Ok(true)
    }

    pub async fn clear_queue(&self, queue: QueueName) -> Result<u32, Box<dyn Error + Send + Sync>> {
        Ok(self.channel.queue_purge(&queue.to_string(), QueuePurgeOptions::default()).await?)
    }

    // pub async fn publish_to_exchange<T>(&self, exchange: ExchangeName, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    // where
    //     T: serde::Serialize,
    // {
    //     self.publish_message(&exchange.to_string(), "", message).await
    // }
}
