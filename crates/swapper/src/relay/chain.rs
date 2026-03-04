use primitives::{Chain, chain_evm::EVMChain};

const BITCOIN_CHAIN_ID: u64 = 8253038;
const SOLANA_CHAIN_ID: u64 = 792703809;

pub enum RelayChain {
    Bitcoin,
    Solana,
    Evm(EVMChain),
}

impl RelayChain {
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

    pub fn to_chain(&self) -> Chain {
        match self {
            Self::Bitcoin => Chain::Bitcoin,
            Self::Solana => Chain::Solana,
            Self::Evm(evm_chain) => evm_chain.to_chain(),
        }
    }
}
