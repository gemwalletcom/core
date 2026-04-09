use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WebhookKind {
    Transactions,
    Support,
    SupportBot,
    Fiat,
}
