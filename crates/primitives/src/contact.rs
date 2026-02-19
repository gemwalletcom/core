use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub struct ContactAddress {
    pub id: String,
    pub contact_id: String,
    pub address: String,
    pub chain: Chain,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ContactData {
    pub contact: Contact,
    pub addresses: Vec<ContactAddress>,
}
