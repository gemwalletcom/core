use primitives::BitcoinChain;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct BitcoinChainConfig {
    blocks_fee_priority: BlocksFeePriority,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct BlocksFeePriority {
    normal: i32,
    slow: i32,
    fast: i32,
}

pub fn get_bitcoin_chain_config(chain: BitcoinChain) -> BitcoinChainConfig {
    BitcoinChainConfig {
        blocks_fee_priority: BitcoinChainConfig::get_blocks_fee_priority(chain),
    }
}

impl BitcoinChainConfig {
    fn get_blocks_fee_priority(chain: BitcoinChain) -> BlocksFeePriority {
        match chain {
            BitcoinChain::Bitcoin => BlocksFeePriority {
                slow: 6,
                normal: 3,
                fast: 1,
            },
            BitcoinChain::Litecoin => BlocksFeePriority {
                slow: 6,
                normal: 3,
                fast: 1,
            },
            BitcoinChain::Doge => BlocksFeePriority {
                slow: 8,
                normal: 4,
                fast: 2,
            },
        }
    }
}
