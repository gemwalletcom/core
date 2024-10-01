use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::chain::Chain;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct Subscription {
    pub wallet_index: i32,
    pub chain: Chain,
    pub address: String,
}
