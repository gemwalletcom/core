use primitives::StakeChain;
use std::str::FromStr;

pub fn get_stake_lock_time(chain: &str) -> u64 {
    match StakeChain::from_str(chain) {
        Ok(chain) => chain.get_lock_time(),
        Err(_) => 0,
    }
}

pub fn get_min_stake_amount(chain: &str) -> u64 {
    match StakeChain::from_str(chain) {
        Ok(chain) => chain.get_min_stake_amount(),
        Err(_) => 0,
    }
}
