use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[derive(Default)]
pub enum PaymentType {
    #[default]
    Card, // debit / credit card
    GooglePay,
    ApplePay,
}

impl PaymentType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}
