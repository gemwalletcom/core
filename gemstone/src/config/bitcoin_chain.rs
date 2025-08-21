use primitives::{chain_bitcoin, BitcoinChain};

// TODO: Gateway. Delete
#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct BitcoinChainConfig {
    blocks_fee_priority: BlocksFeePriority,
    minimum_byte_fee: i32,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct BlocksFeePriority {
    normal: i32,
    slow: i32,
    fast: i32,
}

pub fn get_bitcoin_chain_config(chain: BitcoinChain) -> BitcoinChainConfig {
    BitcoinChainConfig {
        blocks_fee_priority: chain.get_blocks_fee_priority().into(),
        minimum_byte_fee: chain.minimum_byte_fee(),
    }
}

impl From<chain_bitcoin::BlocksFeePriority> for BlocksFeePriority {
    fn from(value: chain_bitcoin::BlocksFeePriority) -> Self {
        BlocksFeePriority {
            normal: value.normal,
            slow: value.slow,
            fast: value.fast,
        }
    }
}
