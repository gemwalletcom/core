use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::chain::Chain;
use crate::chain_address::ChainAddress;
use crate::device::Device;
use crate::wallet::WalletSource;
use crate::wallet_id::WalletId;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct Subscription {
    pub wallet_index: i32,
    pub chain: Chain,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct WalletSubscription {
    #[typeshare(serialized_as = "String")]
    pub wallet_id: WalletId,
    #[serde(default)]
    pub source: Option<WalletSource>,
    pub subscriptions: Vec<ChainAddress>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct WalletSubscriptionChains {
    #[typeshare(serialized_as = "String")]
    pub wallet_id: WalletId,
    pub chains: Vec<Chain>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSubscription {
    pub device: Device,
    pub subscription: Subscription,
}
