use primitives::{Chain, chain_evm::EVMChain};

pub const BITCOIN_CHAIN_ID: u64 = 8253038;
pub const SOLANA_CHAIN_ID: u64 = 792703809;
pub const BITCOIN_CURRENCY: &str = "bc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmql8k8";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayChain {
    Bitcoin,
    Ethereum,
    Solana,
    SmartChain,
    Base,
    Arbitrum,
    Hyperliquid,
    Berachain,
    Manta,
    Sonic,
    Abstract,
    Mantle,
    Celo,
    Stable,
}

impl RelayChain {
    pub fn chain_id(&self) -> u64 {
        match self {
            Self::Bitcoin => BITCOIN_CHAIN_ID,
            Self::Solana => SOLANA_CHAIN_ID,
            _ => EVMChain::from_chain(self.to_chain()).unwrap().chain_id(),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        match chain {
            Chain::Bitcoin => Some(Self::Bitcoin),
            Chain::Ethereum => Some(Self::Ethereum),
            Chain::Solana => Some(Self::Solana),
            Chain::SmartChain => Some(Self::SmartChain),
            Chain::Base => Some(Self::Base),
            Chain::Arbitrum => Some(Self::Arbitrum),
            Chain::Hyperliquid => Some(Self::Hyperliquid),
            Chain::Berachain => Some(Self::Berachain),
            Chain::Manta => Some(Self::Manta),
            Chain::Sonic => Some(Self::Sonic),
            Chain::Abstract => Some(Self::Abstract),
            Chain::Mantle => Some(Self::Mantle),
            Chain::Celo => Some(Self::Celo),
            Chain::Stable => Some(Self::Stable),
            _ => None,
        }
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Bitcoin => Chain::Bitcoin,
            Self::Ethereum => Chain::Ethereum,
            Self::Solana => Chain::Solana,
            Self::SmartChain => Chain::SmartChain,
            Self::Base => Chain::Base,
            Self::Arbitrum => Chain::Arbitrum,
            Self::Hyperliquid => Chain::Hyperliquid,
            Self::Berachain => Chain::Berachain,
            Self::Manta => Chain::Manta,
            Self::Sonic => Chain::Sonic,
            Self::Abstract => Chain::Abstract,
            Self::Mantle => Chain::Mantle,
            Self::Celo => Chain::Celo,
            Self::Stable => Chain::Stable,
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            BITCOIN_CHAIN_ID => Some(Self::Bitcoin),
            SOLANA_CHAIN_ID => Some(Self::Solana),
            _ => {
                let chain = EVMChain::all().into_iter().find(|c| c.chain_id() == chain_id)?.to_chain();
                Self::from_chain(&chain)
            }
        }
    }

    pub fn is_evm(&self) -> bool {
        EVMChain::from_chain(self.to_chain()).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_chain() {
        assert_eq!(RelayChain::from_chain(&Chain::Ethereum), Some(RelayChain::Ethereum));
        assert_eq!(RelayChain::from_chain(&Chain::Solana), Some(RelayChain::Solana));
        assert_eq!(RelayChain::from_chain(&Chain::SmartChain), Some(RelayChain::SmartChain));
        assert_eq!(RelayChain::from_chain(&Chain::Cosmos), None);
    }
}
