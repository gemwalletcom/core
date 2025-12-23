use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::chain_config::EvmChainConfig;
use crate::Chain;

pub use crate::chain_config::ChainStack;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum EVMChain {
    Ethereum,
    SmartChain,
    Polygon,
    Plasma,
    Arbitrum,
    Optimism,
    Base,
    AvalancheC,
    OpBNB,
    Fantom,
    Gnosis,
    Manta,
    Blast,
    ZkSync,
    Linea,
    Mantle,
    Celo,
    World,
    Sonic,
    Abstract,
    Berachain,
    Ink,
    Unichain,
    Hyperliquid,
    Monad,
    XLayer,
    Stable,
}

impl EVMChain {
    fn config(&self) -> &'static EvmChainConfig {
        let chain = self.to_chain();
        let config = chain.config();
        config
            .evm
            .as_ref()
            .unwrap_or_else(|| panic!("Missing EVM config for {chain}"))
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn min_priority_fee(&self) -> u64 {
        self.config().min_priority_fee
    }

    pub fn chain_stack(&self) -> ChainStack {
        self.config().chain_stack
    }

    pub fn is_ethereum_layer2(&self) -> bool {
        self.config().is_ethereum_layer2
    }

    // https://docs.optimism.io/stack/getting-started
    pub fn is_opstack(&self) -> bool {
        self.chain_stack() == ChainStack::Optimism
    }

    // https://docs.zksync.io/zk-stack/running/quickstart
    pub fn is_zkstack(&self) -> bool {
        self.chain_stack() == ChainStack::ZkSync
    }

    pub fn weth_contract(&self) -> Option<&str> {
        self.config().weth_contract
    }

    pub fn from_chain(chain: Chain) -> Option<Self> {
        EVMChain::from_str(chain.as_ref()).ok()
    }

    pub fn to_chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Chain, EVMChain};

    #[test]
    fn test_from_chain() {
        assert_eq!(EVMChain::from_chain(Chain::Ethereum), Some(EVMChain::Ethereum));
        assert_eq!(EVMChain::from_chain(Chain::Bitcoin), None);
    }
}
