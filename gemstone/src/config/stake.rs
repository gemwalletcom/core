use primitives::StakeChain;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct StakeChainConfig {
    pub time_lock: u64,
    pub min_amount: u64,
    pub change_amount_on_unstake: bool,
    pub redelegate: bool,
}

pub fn get_stake_config(chain: StakeChain) -> StakeChainConfig {
    StakeChainConfig {
        time_lock: chain.get_lock_time(),
        min_amount: chain.get_min_stake_amount(),
        change_amount_on_unstake: chain.get_change_amount_on_unstake(),
        redelegate: chain.get_redelegate(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stake_config() {
        assert_eq!(
            get_stake_config(StakeChain::Sui),
            StakeChainConfig {
                time_lock: 86400,
                min_amount: 1000000000,
                change_amount_on_unstake: false,
                redelegate: false,
            }
        );
    }
}
