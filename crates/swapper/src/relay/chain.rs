use primitives::{Chain, chain_evm::EVMChain};

const BITCOIN_CHAIN_ID: u64 = 8253038;
const SOLANA_CHAIN_ID: u64 = 792703809;

pub enum RelayChain {
    Bitcoin,
    Solana,
    Evm(EVMChain),
}

impl RelayChain {
    pub fn chain_id(&self) -> u64 {
        match self {
            Self::Bitcoin => BITCOIN_CHAIN_ID,
            Self::Solana => SOLANA_CHAIN_ID,
            Self::Evm(evm_chain) => evm_chain.chain_id(),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        match chain {
            Chain::Bitcoin => Some(Self::Bitcoin),
            Chain::Solana => Some(Self::Solana),
            _ => Some(Self::Evm(EVMChain::from_chain(*chain)?)),
        }
    }

    pub fn to_chain(&self) -> Chain {
        match self {
            Self::Bitcoin => Chain::Bitcoin,
            Self::Solana => Chain::Solana,
            Self::Evm(evm_chain) => evm_chain.to_chain(),
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            BITCOIN_CHAIN_ID => Some(Self::Bitcoin),
            SOLANA_CHAIN_ID => Some(Self::Solana),
            _ => {
                let evm_chain = EVMChain::all().into_iter().find(|c| c.chain_id() == chain_id)?;
                Some(Self::Evm(evm_chain))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_chain() {
        assert_eq!(RelayChain::from_chain(&Chain::Ethereum).unwrap().chain_id(), EVMChain::Ethereum.chain_id());
        assert_eq!(RelayChain::from_chain(&Chain::SmartChain).unwrap().chain_id(), EVMChain::SmartChain.chain_id());
        assert_eq!(RelayChain::from_chain(&Chain::Solana).unwrap().chain_id(), SOLANA_CHAIN_ID);
        assert!(RelayChain::from_chain(&Chain::Cosmos).is_none());
    }
}
