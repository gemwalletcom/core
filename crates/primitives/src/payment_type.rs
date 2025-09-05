use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PaymentType {
    Card, // debit / credit card
    GooglePay,
    ApplePay,
}

impl Default for PaymentType {
    fn default() -> Self {
        Self::Card
    }
}
