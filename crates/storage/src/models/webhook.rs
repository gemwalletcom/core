use diesel::prelude::*;
use primitives::WebhookKind;

use crate::sql_types::WebhookKind as WebhookKindRow;

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::webhook_endpoints)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewWebhookEndpointRow {
    pub kind: WebhookKindRow,
    pub sender: String,
}

impl NewWebhookEndpointRow {
    pub fn new(kind: WebhookKind, sender: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            sender: sender.into(),
        }
    }
}
