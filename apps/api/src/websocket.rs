use std::error::Error;

use redis::aio::MultiplexedConnection;
use redis::{PushInfo, PushKind};
use rocket_ws::stream::DuplexStream;
use tokio::sync::mpsc::UnboundedReceiver;

pub fn decode_push_message(message: &PushInfo) -> Option<(&str, &[u8])> {
    match (&message.kind, message.data.as_slice()) {
        (PushKind::Message, [redis::Value::BulkString(channel), redis::Value::BulkString(value)]) => Some((std::str::from_utf8(channel).ok()?, value)),
        _ => None,
    }
}

pub async fn setup_ws_resources(redis_url: &str, stream: DuplexStream) -> Result<(DuplexStream, MultiplexedConnection, UnboundedReceiver<PushInfo>), Box<dyn Error + Send + Sync>> {
    let client = redis::Client::open(redis_url)?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);
    let redis_connection = client.get_multiplexed_async_connection_with_config(&config).await?;
    Ok((stream, redis_connection, rx))
}
