use crate::{SwapperError, mayan::wormhole_chain::WormholeChain};

pub(in crate::mayan) const CCTP_TOKEN_DECIMALS: u32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(in crate::mayan) enum CCTPDomain {
    Ethereum = 0,
    Avalanche = 1,
    Optimism = 2,
    Arbitrum = 3,
    Solana = 5,
    Base = 6,
    Polygon = 7,
    Sui = 8,
    Unichain = 10,
    Linea = 11,
    Sonic = 13,
    Monad = 15,
    Hyperevm = 19,
}

impl CCTPDomain {
    pub(in crate::mayan) const fn id(self) -> u32 {
        self as u32
    }
}

impl TryFrom<WormholeChain> for CCTPDomain {
    type Error = SwapperError;

    fn try_from(chain: WormholeChain) -> Result<Self, Self::Error> {
        match chain {
            WormholeChain::Ethereum => Ok(Self::Ethereum),
            WormholeChain::Avalanche => Ok(Self::Avalanche),
            WormholeChain::Optimism => Ok(Self::Optimism),
            WormholeChain::Arbitrum => Ok(Self::Arbitrum),
            WormholeChain::Solana => Ok(Self::Solana),
            WormholeChain::Base => Ok(Self::Base),
            WormholeChain::Polygon => Ok(Self::Polygon),
            WormholeChain::Sui => Ok(Self::Sui),
            WormholeChain::Unichain => Ok(Self::Unichain),
            WormholeChain::Linea => Ok(Self::Linea),
            WormholeChain::Sonic => Ok(Self::Sonic),
            WormholeChain::Monad => Ok(Self::Monad),
            WormholeChain::Hyperevm => Ok(Self::Hyperevm),
            _ => Err(SwapperError::NotSupportedChain),
        }
    }
}

pub(in crate::mayan) fn domain_for_wormhole_chain(chain: &str) -> Result<CCTPDomain, SwapperError> {
    CCTPDomain::try_from(WormholeChain::from_name(chain)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_for_wormhole_chain() {
        assert_eq!(domain_for_wormhole_chain(WormholeChain::Ethereum.name()).unwrap(), CCTPDomain::Ethereum);
        assert_eq!(domain_for_wormhole_chain(WormholeChain::Base.name()).unwrap(), CCTPDomain::Base);
        assert_eq!(domain_for_wormhole_chain(WormholeChain::Sui.name()).unwrap(), CCTPDomain::Sui);
        assert_eq!(domain_for_wormhole_chain(WormholeChain::Solana.name()).unwrap(), CCTPDomain::Solana);
        assert_eq!(domain_for_wormhole_chain(WormholeChain::Hyperevm.name()).unwrap(), CCTPDomain::Hyperevm);
        assert_eq!(domain_for_wormhole_chain("bitcoin").unwrap_err(), SwapperError::NotSupportedChain);
    }
}
