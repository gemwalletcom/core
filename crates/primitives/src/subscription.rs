use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::chain::Chain;
use crate::chain_address::ChainAddress;
use crate::device::Device;
use crate::wallet::WalletSource;
use crate::wallet_id::WalletId;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AddressChains {
    pub address: String,
    pub chains: Vec<Chain>,
}

impl AddressChains {
    pub fn new(address: String, chains: Vec<Chain>) -> Self {
        Self {
            address,
            chains: chains.into_iter().collect::<BTreeSet<_>>().into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletSubscription {
    #[typeshare(serialized_as = "String")]
    pub wallet_id: WalletId,
    #[serde(default)]
    pub source: Option<WalletSource>,
    pub subscriptions: Vec<AddressChains>,
}

impl WalletSubscription {
    pub fn chain_addresses(&self) -> Vec<ChainAddress> {
        let mut seen = BTreeSet::new();
        self.subscriptions
            .iter()
            .flat_map(|x| x.chains.iter().map(|&chain| ChainAddress::new(chain, x.address.clone())))
            .filter(|x| seen.insert((x.chain, x.address.clone())))
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletSubscriptionLegacy {
    pub wallet_id: WalletId,
    #[serde(default)]
    pub source: Option<WalletSource>,
    pub subscriptions: Vec<ChainAddress>,
}

impl From<WalletSubscriptionLegacy> for WalletSubscription {
    fn from(legacy: WalletSubscriptionLegacy) -> Self {
        let mut by_address: BTreeMap<String, Vec<Chain>> = BTreeMap::new();
        for subscription in &legacy.subscriptions {
            by_address.entry(subscription.address.clone()).or_default().push(subscription.chain);
        }
        Self {
            wallet_id: legacy.wallet_id,
            source: legacy.source,
            subscriptions: by_address.into_iter().map(|(address, chains)| AddressChains::new(address, chains)).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletSubscriptionChains {
    #[typeshare(serialized_as = "String")]
    pub wallet_id: WalletId,
    pub chains: Vec<Chain>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSubscription {
    pub device: Device,
    pub wallet_id: WalletId,
    pub chain: Chain,
    pub address: String,
}
