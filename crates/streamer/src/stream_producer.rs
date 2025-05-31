use std::error::Error;

use lapin::{options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties};

use crate::QueueName;

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
    pub async fn publish<T>(&self, queue: QueueName, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let data = serde_json::to_vec(message)?;
        let confirm = self
            .channel
            .basic_publish(
                "", // Default exchange
                &queue.to_string(),
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
}
