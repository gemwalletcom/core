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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stake_lock_time() {
        assert_eq!(get_stake_lock_time("sui"), 86400);
        assert_eq!(get_stake_lock_time("smartchain"), 604800);
    }

    #[test]
    fn test_get_min_stake_amount() {
        assert_eq!(get_min_stake_amount("sui"), 1_000_000_000);
        assert_eq!(
            get_min_stake_amount("smartchain"),
            1_000_000_000_000_000_000
        );
    }
}
