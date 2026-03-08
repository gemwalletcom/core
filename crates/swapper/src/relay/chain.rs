use primitives::{Chain, chain_evm::EVMChain};

pub const BITCOIN_CHAIN_ID: u64 = 8253038;
pub const BITCOIN_CURRENCY: &str = "bc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmql8k8";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayChain {
    Bitcoin,
    Evm(EVMChain),
}

impl RelayChain {
    pub fn chain_id(&self) -> u64 {
        match self {
            Self::Bitcoin => BITCOIN_CHAIN_ID,
            Self::Evm(chain) => chain.chain_id(),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        if *chain == Chain::Bitcoin {
            return Some(Self::Bitcoin);
        }
        Some(Self::Evm(EVMChain::from_chain(*chain)?))
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Bitcoin => Chain::Bitcoin,
            Self::Evm(chain) => chain.to_chain(),
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        if chain_id == BITCOIN_CHAIN_ID {
            return Some(Self::Bitcoin);
        }
        Some(Self::Evm(EVMChain::all().into_iter().find(|chain| chain.chain_id() == chain_id)?))
    }

    pub fn is_evm(&self) -> bool {
        matches!(self, Self::Evm(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_chain() {
        assert_eq!(RelayChain::from_chain(&Chain::Ethereum).unwrap().chain_id(), EVMChain::Ethereum.chain_id());
        assert_eq!(RelayChain::from_chain(&Chain::SmartChain).unwrap().chain_id(), EVMChain::SmartChain.chain_id());
        assert_eq!(RelayChain::from_chain(&Chain::Bitcoin), Some(RelayChain::Bitcoin));
        assert!(RelayChain::from_chain(&Chain::Solana).is_none());
        assert!(RelayChain::from_chain(&Chain::Cosmos).is_none());
    }

    #[test]
    fn test_from_chain_id() {
        assert_eq!(RelayChain::from_chain_id(BITCOIN_CHAIN_ID), Some(RelayChain::Bitcoin));
        assert_eq!(RelayChain::from_chain_id(1).unwrap().to_chain(), Chain::Ethereum);
        assert!(RelayChain::from_chain_id(999999999).is_none());
    }
}
