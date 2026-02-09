use std::error::Error;
use std::sync::Arc;

use lapin::{Channel, Connection, ConnectionProperties};

#[derive(Clone)]
pub struct StreamConnection {
    url: String,
    name: String,
    connection: Arc<Connection>,
}

impl StreamConnection {
    pub async fn new(url: &str, name: impl Into<String>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let name: String = name.into();
        let connection = Connection::connect(url, ConnectionProperties::default().with_connection_name(name.clone().into())).await?;
        Ok(Self {
            url: url.to_string(),
            name,
            connection: Arc::new(connection),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn create_channel(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        Ok(self.connection.create_channel().await?)
    }
}
