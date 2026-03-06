use primitives::{Chain, chain_evm::EVMChain};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayChain {
    Evm(EVMChain),
}

impl RelayChain {
    pub fn chain_id(&self) -> u64 {
        match self {
            Self::Evm(chain) => chain.chain_id(),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        Some(Self::Evm(EVMChain::from_chain(*chain)?))
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Evm(chain) => chain.to_chain(),
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        Some(Self::Evm(EVMChain::all().into_iter().find(|chain| chain.chain_id() == chain_id)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_chain() {
        assert_eq!(RelayChain::from_chain(&Chain::Ethereum).unwrap().chain_id(), EVMChain::Ethereum.chain_id());
        assert_eq!(RelayChain::from_chain(&Chain::SmartChain).unwrap().chain_id(), EVMChain::SmartChain.chain_id());
        assert!(RelayChain::from_chain(&Chain::Solana).is_none());
        assert!(RelayChain::from_chain(&Chain::Bitcoin).is_none());
        assert!(RelayChain::from_chain(&Chain::Cosmos).is_none());
    }
}
