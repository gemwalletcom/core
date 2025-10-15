pub mod constants;
pub mod contracts;
pub mod mapper;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub mod queue;
#[cfg(feature = "rpc")]
pub mod state;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub mod stats;

pub use constants::*;
pub use contracts::*;
pub use mapper::*;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub use queue::*;

#[cfg(all(feature = "rpc", not(feature = "reqwest")))]
use primitives::StakeChain;
#[cfg(feature = "rpc")]
use primitives::StakeLockTime;
#[cfg(feature = "rpc")]
use std::error::Error;

#[cfg(feature = "rpc")]
pub async fn get_everstake_lock_time() -> Result<StakeLockTime, Box<dyn Error + Send + Sync>> {
    #[cfg(feature = "reqwest")]
    {
        let queue = queue::get_everstake_validator_queue().await?;
        let activation_time = queue.validator_activation_time + queue.validator_adding_delay;
        let withdrawal_time = queue.validator_exit_time + queue.validator_withdraw_time;

        Ok(StakeLockTime::new(withdrawal_time, Some(activation_time)))
    }

    #[cfg(not(feature = "reqwest"))]
    {
        Ok(StakeLockTime::new(StakeChain::Ethereum.get_lock_time(), None))
    }
}
