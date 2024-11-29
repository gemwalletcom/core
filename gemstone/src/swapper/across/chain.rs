use primitives::Chain;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, EnumIter)]
pub enum AcrossChainName {
    Arbitrum,
    Base,
    Blast,
    Ethereum,
    Linea,
    Optimism,
    Polygon,
    World,
    ZkSync,
    // Mode,
    // Redstone,
    // Scroll,
    // Lisk,
    //Zora,
}

// https://dev.thorchain.org/concepts/memo-length-reduction.html
impl AcrossChainName {
    pub fn all() -> Vec<AcrossChainName> {
        AcrossChainName::iter().collect::<Vec<_>>()
    }
    pub fn short_name(&self) -> &str {
        match self {
            AcrossChainName::Arbitrum => "a", //
            AcrossChainName::Base => "ba",    //
            AcrossChainName::Blast => "bl",   // ""
            AcrossChainName::Ethereum => "e", //
            AcrossChainName::Linea => "l",    //
            AcrossChainName::Optimism => "a", //
            AcrossChainName::Polygon => "a",  //
            AcrossChainName::World => "w",    //
            AcrossChainName::ZkSync => "a",   //
        }
    }

    pub fn long_name(&self) -> &str {
        match self {
            AcrossChainName::Arbitrum => "Arbitrum", //
            AcrossChainName::Base => "BASE",         //
            AcrossChainName::Blast => "BLAST",       // ""
            AcrossChainName::Ethereum => "Ethereum", //
            AcrossChainName::Linea => "Linea",       //
            AcrossChainName::Optimism => "Optimism", //
            AcrossChainName::Polygon => "Polygon",   //
            AcrossChainName::World => "WorldChain",  //
            AcrossChainName::ZkSync => "ZkSync",     //
        }
    }

    pub fn chain(&self) -> Chain {
        match self {
            AcrossChainName::Arbitrum => Chain::Arbitrum, //
            AcrossChainName::Base => Chain::Base,         //
            AcrossChainName::Blast => Chain::Blast,       // ""
            AcrossChainName::Ethereum => Chain::Ethereum, //
            AcrossChainName::Linea => Chain::Linea,       //
            AcrossChainName::Optimism => Chain::Optimism, //
            AcrossChainName::Polygon => Chain::Polygon,   //
            AcrossChainName::World => Chain::World,       //
            AcrossChainName::ZkSync => Chain::ZkSync,     //
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<AcrossChainName> {
        match chain {
            Chain::Arbitrum => Some(AcrossChainName::Arbitrum),
            Chain::Base => Some(AcrossChainName::Base),
            Chain::Blast => Some(AcrossChainName::Blast),
            Chain::Ethereum => Some(AcrossChainName::Ethereum),
            Chain::Linea => Some(AcrossChainName::Linea),
            Chain::Optimism => Some(AcrossChainName::Optimism),
            Chain::Polygon => Some(AcrossChainName::Polygon),
            Chain::World => Some(AcrossChainName::World),
            Chain::ZkSync => Some(AcrossChainName::ZkSync),
            _ => None,
        }
    }
}
