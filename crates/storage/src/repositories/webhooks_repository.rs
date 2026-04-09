use crate::database::webhooks::WebhooksStore;
use crate::models::NewWebhookEndpointRow;
use crate::{DatabaseClient, DatabaseError};
use primitives::WebhookKind;

pub trait WebhooksRepository {
    fn add_webhook_endpoints(&mut self, values: Vec<NewWebhookEndpointRow>) -> Result<usize, DatabaseError>;
    fn get_webhook_endpoint(&mut self, kind: WebhookKind, sender: &str, secret: &str) -> Result<Option<bool>, DatabaseError>;
}

impl WebhooksRepository for DatabaseClient {
    fn add_webhook_endpoints(&mut self, values: Vec<NewWebhookEndpointRow>) -> Result<usize, DatabaseError> {
        Ok(WebhooksStore::add_webhook_endpoints(self, values)?)
    }

    fn get_webhook_endpoint(&mut self, kind: WebhookKind, sender: &str, secret: &str) -> Result<Option<bool>, DatabaseError> {
        Ok(WebhooksStore::get_webhook_endpoint(self, kind, sender, secret)?)
    }
}
