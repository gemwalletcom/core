use primitives::StakeChain;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct StakeChainConfig {
    pub time_lock: u64,
    pub activation_lock_time: u64,
    pub min_amount: u64,
    pub change_amount_on_unstake: bool,
    pub can_redelegate: bool,
    pub can_withdraw: bool,
    pub can_claim_rewards: bool,
    pub reserved_for_fees: u64,
}

pub fn get_stake_config(chain: StakeChain) -> StakeChainConfig {
    let activation_lock_time = match chain {
        StakeChain::Solana => 259200,
        _ => 0,
    };

    StakeChainConfig {
        time_lock: chain.get_lock_time(),
        activation_lock_time,
        min_amount: chain.get_min_stake_amount(),
        change_amount_on_unstake: chain.get_change_amount_on_unstake(),
        can_redelegate: chain.get_can_redelegate(),
        can_withdraw: chain.get_can_withdraw(),
        can_claim_rewards: chain.get_can_claim_rewards(),
        reserved_for_fees: chain.get_reserved_for_fees(),
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
                activation_lock_time: 0,
                min_amount: 1000000000,
                change_amount_on_unstake: false,
                can_redelegate: false,
                can_withdraw: false,
                can_claim_rewards: false,
                reserved_for_fees: 100000000,
            }
        );
    }
}
