use primitives::Chain;

pub const BITCOIN_CHAIN_ID: u64 = 8253038;
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
}

impl RelayChain {
    pub fn chain_id(&self) -> u64 {
        match self {
            RelayChain::Bitcoin => BITCOIN_CHAIN_ID,
            RelayChain::Ethereum => 1,
            RelayChain::Solana => 792703809,
            RelayChain::SmartChain => 56,
            RelayChain::Base => 8453,
            RelayChain::Arbitrum => 42161,
            RelayChain::Hyperliquid => 999,
            RelayChain::Berachain => 80094,
            RelayChain::Manta => 169,
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
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            BITCOIN_CHAIN_ID => Some(RelayChain::Bitcoin),
            1 => Some(RelayChain::Ethereum),
            792703809 => Some(RelayChain::Solana),
            56 => Some(RelayChain::SmartChain),
            8453 => Some(RelayChain::Base),
            42161 => Some(RelayChain::Arbitrum),
            999 => Some(RelayChain::Hyperliquid),
            80094 => Some(RelayChain::Berachain),
            169 => Some(RelayChain::Manta),
            _ => None,
        }
    }

    pub fn is_evm(&self) -> bool {
        match self {
            RelayChain::Bitcoin | RelayChain::Solana => false,
            RelayChain::Ethereum | RelayChain::SmartChain | RelayChain::Base | RelayChain::Arbitrum | RelayChain::Hyperliquid | RelayChain::Berachain | RelayChain::Manta => true,
        }
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
