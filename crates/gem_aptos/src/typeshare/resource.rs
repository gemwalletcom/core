use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosResource<T> {
    pub r#type: String,
    pub data: T,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosResourceBalance {
    pub coin: AptosResourceCoin,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosResourceBalanceOptional {
    pub coin: Option<AptosResourceCoin>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosResourceCoin {
    pub value: String,
}
