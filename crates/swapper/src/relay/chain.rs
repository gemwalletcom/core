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
            RelayChain::Bitcoin => BITCOIN_CHAIN_ID,
            RelayChain::Solana => SOLANA_CHAIN_ID,
            _ => EVMChain::from_chain(self.to_chain()).unwrap().chain_id(),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        match chain {
            Chain::Bitcoin => Some(RelayChain::Bitcoin),
            Chain::Ethereum => Some(RelayChain::Ethereum),
            Chain::Solana => Some(RelayChain::Solana),
            Chain::SmartChain => Some(RelayChain::SmartChain),
            Chain::Base => Some(RelayChain::Base),
            Chain::Arbitrum => Some(RelayChain::Arbitrum),
            Chain::Hyperliquid => Some(RelayChain::Hyperliquid),
            Chain::Berachain => Some(RelayChain::Berachain),
            Chain::Manta => Some(RelayChain::Manta),
            Chain::Sonic => Some(RelayChain::Sonic),
            Chain::Abstract => Some(RelayChain::Abstract),
            Chain::Mantle => Some(RelayChain::Mantle),
            Chain::Celo => Some(RelayChain::Celo),
            Chain::Stable => Some(RelayChain::Stable),
            _ => None,
        }
    }

    pub fn to_chain(self) -> Chain {
        match self {
            RelayChain::Bitcoin => Chain::Bitcoin,
            RelayChain::Ethereum => Chain::Ethereum,
            RelayChain::Solana => Chain::Solana,
            RelayChain::SmartChain => Chain::SmartChain,
            RelayChain::Base => Chain::Base,
            RelayChain::Arbitrum => Chain::Arbitrum,
            RelayChain::Hyperliquid => Chain::Hyperliquid,
            RelayChain::Berachain => Chain::Berachain,
            RelayChain::Manta => Chain::Manta,
            RelayChain::Sonic => Chain::Sonic,
            RelayChain::Abstract => Chain::Abstract,
            RelayChain::Mantle => Chain::Mantle,
            RelayChain::Celo => Chain::Celo,
            RelayChain::Stable => Chain::Stable,
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            BITCOIN_CHAIN_ID => Some(RelayChain::Bitcoin),
            SOLANA_CHAIN_ID => Some(RelayChain::Solana),
            _ => Self::ALL_EVM.iter().find(|c| c.chain_id() == chain_id).copied(),
        }
    }

    pub fn is_evm(&self) -> bool {
        !matches!(self, RelayChain::Bitcoin | RelayChain::Solana)
    }

    const ALL_EVM: [RelayChain; 12] = [
        RelayChain::Ethereum,
        RelayChain::SmartChain,
        RelayChain::Base,
        RelayChain::Arbitrum,
        RelayChain::Hyperliquid,
        RelayChain::Berachain,
        RelayChain::Manta,
        RelayChain::Sonic,
        RelayChain::Abstract,
        RelayChain::Mantle,
        RelayChain::Celo,
        RelayChain::Stable,
    ];
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
