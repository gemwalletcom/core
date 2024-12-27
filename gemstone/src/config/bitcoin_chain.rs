use primitives::BitcoinChain;

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
        blocks_fee_priority: BitcoinChainConfig::get_blocks_fee_priority(chain),
        minimum_byte_fee: BitcoinChainConfig::minimum_byte_fee(chain),
    }
}

impl BitcoinChainConfig {
    fn get_blocks_fee_priority(chain: BitcoinChain) -> BlocksFeePriority {
        match chain {
            BitcoinChain::Bitcoin => BlocksFeePriority { slow: 6, normal: 3, fast: 1 },
            BitcoinChain::BitcoinCash => BlocksFeePriority { slow: 6, normal: 3, fast: 1 },
            BitcoinChain::Litecoin => BlocksFeePriority { slow: 6, normal: 3, fast: 1 },
            BitcoinChain::Doge => BlocksFeePriority { slow: 8, normal: 4, fast: 2 },
        }
    }

    fn minimum_byte_fee(chain: BitcoinChain) -> i32 {
        match chain {
            BitcoinChain::Bitcoin => 1,
            BitcoinChain::BitcoinCash => 5,
            BitcoinChain::Litecoin => 5,
            BitcoinChain::Doge => 1000,
        }
    }
}
