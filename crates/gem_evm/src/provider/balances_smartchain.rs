use crate::rpc::client::EthereumClient;
use gem_client::Client;
use num_bigint::BigUint;
use primitives::{AssetBalance, Balance};
use std::error::Error;
use std::str::FromStr;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_smartchain_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let (delegations, undelegations) = self.fetch_smartchain_staking_state(address).await?;

        let staked = delegations
            .iter()
            .filter_map(|d| BigUint::from_str(&d.amount).ok())
            .fold(BigUint::from(0u32), |acc, amount| acc + amount);

        let pending = undelegations
            .iter()
            .filter_map(|u| BigUint::from_str(&u.amount).ok())
            .fold(BigUint::from(0u32), |acc, amount| acc + amount);

        Ok(Some(AssetBalance::new_balance(
            self.get_chain().as_asset_id(),
            Balance::stake_balance(staked, pending, None),
        )))
    }
}
