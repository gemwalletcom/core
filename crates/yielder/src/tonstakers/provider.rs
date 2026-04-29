use std::sync::Arc;

use async_trait::async_trait;
use futures::try_join;
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType, YieldProvider};

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use gem_ton::{
    Address as TonAddress,
    models::JettonWallet,
    rpc::client::TonClient,
    tonstakers::{STAKING_CONTRACT, TS_TON_MASTER, build_stake_payload_base64, build_unstake_payload_base64, get_pool_full_data},
};

use crate::client_factory::create_chain_client;
use crate::error::YielderError;
use crate::provider::EarnProvider;

use super::mapper::{map_redeem_shares, map_staked_balance, map_to_delegation};

pub struct TonstakersProvider {
    rpc_provider: Arc<dyn RpcProvider>,
}

impl TonstakersProvider {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self { rpc_provider }
    }

    fn get_client(&self) -> Result<TonClient<RpcClient>, YielderError> {
        Ok(TonClient::new(create_chain_client(self.rpc_provider.clone(), Chain::Ton)?))
    }

    fn parse_owner(address: &str) -> Result<TonAddress, YielderError> {
        TonAddress::parse(address).map_err(|err| YielderError::invalid_input(err.to_string()))
    }

    fn find_tonstaker_wallet(wallets: Vec<JettonWallet>) -> Option<JettonWallet> {
        wallets.into_iter().find(|wallet| wallet.jetton == TS_TON_MASTER)
    }
}

#[async_trait]
impl EarnProvider for TonstakersProvider {
    fn get_provider(&self, asset_id: &AssetId) -> Option<DelegationValidator> {
        (asset_id.chain == Chain::Ton && asset_id.is_native()).then(|| YieldProvider::Tonstakers.delegation_validator(Chain::Ton))
    }

    async fn get_position(&self, address: &str, _asset_id: &AssetId) -> Result<Option<DelegationBase>, YielderError> {
        let client = self.get_client()?;
        let owner = Self::parse_owner(address)?;
        let jetton_wallets = client.get_jetton_wallets(owner.to_string()).await?;
        let wallet = Self::find_tonstaker_wallet(jetton_wallets.jetton_wallets).filter(|wallet| !wallet.balance.eq(&BigUint::ZERO));

        let Some(wallet) = wallet else {
            return Ok(None);
        };

        let pool = get_pool_full_data(&client, STAKING_CONTRACT).await?;
        let balance = map_staked_balance(&wallet.balance, &pool.total_balance, &pool.supply);

        Ok(Some(map_to_delegation(balance, wallet.balance)))
    }

    async fn get_balance(&self, _chain: Chain, address: &str, _token_ids: &[String]) -> Result<Vec<AssetBalance>, YielderError> {
        Ok(self
            .get_position(address, &AssetId::from_chain(Chain::Ton))
            .await?
            .map(|delegation| AssetBalance::new_earn(delegation.asset_id, delegation.balance))
            .into_iter()
            .collect())
    }

    async fn get_data(&self, _asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError> {
        let owner = Self::parse_owner(address)?;
        let amount = BigUint::parse_bytes(value.as_bytes(), 10).ok_or_else(|| YielderError::invalid_input("invalid TON amount"))?;

        match earn_type {
            EarnType::Deposit(_) => Ok(ContractCallData::new(STAKING_CONTRACT.to_string(), build_stake_payload_base64()?)),
            EarnType::Withdraw(delegation) => {
                let client = self.get_client()?;
                let (jetton_wallets, pool) = try_join!(client.get_jetton_wallets(owner.to_string()), get_pool_full_data(&client, STAKING_CONTRACT))?;
                let wallet = Self::find_tonstaker_wallet(jetton_wallets.jetton_wallets).ok_or_else(|| YielderError::invalid_input("missing Tonstakers jetton wallet"))?;
                let redeem_shares = map_redeem_shares(&amount, &pool.total_balance, &pool.supply, &delegation.base.shares);

                Ok(ContractCallData::new(wallet.address, build_unstake_payload_base64(&owner, &redeem_shares)?))
            }
        }
    }
}

#[cfg(all(test, feature = "yield_integration_tests"))]
mod integration_tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use primitives::{AssetId, Chain, DelegationState, YieldProvider};
    use settings::testkit::get_test_settings;
    use swapper::NativeProvider;

    use super::TonstakersProvider;
    use crate::error::YielderError;
    use crate::provider::EarnProvider;

    const TONSTAKERS_ADDRESS: &str = "0:DB7FF3D4D432C072778CCD68E351B2BF55CEE86A13F488EA6E1DBC48C037DA23";

    #[tokio::test]
    async fn test_tonstakers_get_balance_and_positions() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let settings = get_test_settings();
        let rpc_provider = Arc::new(NativeProvider::new_with_endpoints(HashMap::from([(Chain::Ton, settings.chains.ton.url)])));
        let asset_id = AssetId::from_chain(Chain::Ton);
        let provider = TonstakersProvider::new(rpc_provider);

        let balances = gem_client::retry(
            || provider.get_balance(Chain::Ton, TONSTAKERS_ADDRESS, &[]),
            3,
            Some(|err: &YielderError| gem_client::default_should_retry(err)),
        )
        .await?;
        println!("balances: {balances:#?}");
        assert_eq!(balances.len(), 1);
        assert_eq!(balances[0].asset_id, asset_id);

        let position = gem_client::retry(
            || provider.get_position(TONSTAKERS_ADDRESS, &asset_id),
            3,
            Some(|err: &YielderError| gem_client::default_should_retry(err)),
        )
        .await?
        .unwrap();
        println!("position: {position:#?}");
        assert_eq!(position.asset_id, asset_id);
        assert_eq!(position.validator_id, YieldProvider::Tonstakers.as_ref());
        assert_eq!(position.state, DelegationState::Active);

        Ok(())
    }
}
