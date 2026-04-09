use diesel::OptionalExtension;
use diesel::prelude::*;
use primitives::WebhookKind;

use crate::DatabaseClient;
use crate::models::NewWebhookEndpointRow;
use crate::sql_types::WebhookKind as WebhookKindRow;

pub trait WebhooksStore {
    fn add_webhook_endpoints(&mut self, values: Vec<NewWebhookEndpointRow>) -> Result<usize, diesel::result::Error>;
    fn get_webhook_endpoint(&mut self, kind: WebhookKind, sender: &str, secret: &str) -> Result<Option<bool>, diesel::result::Error>;
}

impl WebhooksStore for DatabaseClient {
    fn add_webhook_endpoints(&mut self, values: Vec<NewWebhookEndpointRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::webhook_endpoints::dsl::*;

        diesel::insert_into(webhook_endpoints).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn get_webhook_endpoint(&mut self, kind_value: WebhookKind, sender_value: &str, secret_value: &str) -> Result<Option<bool>, diesel::result::Error> {
        use crate::schema::webhook_endpoints::dsl::*;

        webhook_endpoints
            .filter(kind.eq(WebhookKindRow::from(kind_value)))
            .filter(sender.eq(sender_value))
            .filter(secret.eq(secret_value))
            .select(enabled)
            .first(&mut self.connection)
            .optional()
    }
}
