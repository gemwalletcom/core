use num_bigint::BigUint;
use primitives::{AssetAddress, AssetBalance, AssetVecExt, ChainAddress};
use std::collections::HashSet;
use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use settings_chain::ChainProviders;
use storage::{AssetsAddressesRepository, AssetsRepository, Database};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct FetchTokenAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTokenAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Database, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            provider,
            database,
            stream_producer,
            cacher,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTokenAddressesConsumer {
    async fn should_process(&self, payload: ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_cached(CacheKey::FetchTokenAddresses(payload.value.chain.as_ref(), &payload.value.address))
            .await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain_address = payload.value;
        let all_assets = self.provider.get_balance_assets(chain_address.chain, chain_address.address.clone()).await?;
        let mut assets_addresses = self.database.assets_addresses()?;
        let existing_addresses = assets_addresses.get_asset_addresses(chain_address.clone())?;
        let changes = TokenAddressChanges::from_balances(&chain_address, existing_addresses, all_assets);

        let asset_ids: Vec<_> = changes.latest_addresses.iter().map(|address| address.asset_id.clone()).collect();
        let existing_ids: HashSet<_> = self.database.assets()?.get_assets(asset_ids)?.ids().into_iter().collect();
        let mut latest_addresses = Vec::new();
        let mut missing_ids = Vec::new();

        for address in changes.latest_addresses {
            if existing_ids.contains(&address.asset_id) {
                latest_addresses.push(address);
            } else {
                missing_ids.push(address.asset_id);
            }
        }

        let latest_count = latest_addresses.len();
        assets_addresses.delete_assets_addresses(changes.stale_addresses)?;
        assets_addresses.add_assets_addresses(latest_addresses)?;

        self.stream_producer.publish_fetch_assets(missing_ids).await?;

        Ok(latest_count)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TokenAddressChanges {
    latest_addresses: Vec<AssetAddress>,
    stale_addresses: Vec<AssetAddress>,
}

impl TokenAddressChanges {
    fn from_balances(chain_address: &ChainAddress, existing_addresses: Vec<AssetAddress>, latest_balances: Vec<AssetBalance>) -> Self {
        let mut seen = HashSet::new();
        let latest_addresses: Vec<_> = latest_balances
            .into_iter()
            .filter(|asset| asset.asset_id.token_id.is_some())
            .filter(|asset| seen.insert(asset.asset_id.clone()))
            .filter(|asset| asset.balance.available > BigUint::ZERO)
            .map(|asset| AssetAddress::new(asset.asset_id, chain_address.address.clone(), Some(asset.balance.available.to_string())))
            .collect();

        let latest_ids: HashSet<_> = latest_addresses.iter().map(|address| address.asset_id.clone()).collect();
        let stale_addresses = existing_addresses
            .into_iter()
            .filter(|address| address.asset_id.token_id.is_some())
            .filter(|address| !latest_ids.contains(&address.asset_id))
            .collect();

        Self {
            latest_addresses,
            stale_addresses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain};

    #[test]
    fn test_from_balances() {
        let chain_address = ChainAddress::new(Chain::Ethereum, "0xwallet".to_string());
        let existing_addresses = vec![
            AssetAddress::new(Asset::mock_eth().id, chain_address.address.clone(), Some("10".to_string())),
            AssetAddress::new(Asset::mock_ethereum_usdc().id.clone(), chain_address.address.clone(), Some("5".to_string())),
            AssetAddress::new(Asset::mock_erc20().id.clone(), chain_address.address.clone(), Some("7".to_string())),
        ];

        let omitted_zero_changes = TokenAddressChanges::from_balances(
            &chain_address,
            existing_addresses.clone(),
            vec![AssetBalance::new(Asset::mock_erc20().id.clone(), BigUint::from(9u32))],
        );
        assert_eq!(
            omitted_zero_changes.stale_addresses,
            vec![AssetAddress::new(Asset::mock_ethereum_usdc().id, chain_address.address.clone(), Some("5".to_string()))]
        );
        assert_eq!(
            omitted_zero_changes.latest_addresses,
            vec![AssetAddress::new(Asset::mock_erc20().id, chain_address.address.clone(), Some("9".to_string()))]
        );

        let explicit_zero_changes = TokenAddressChanges::from_balances(
            &chain_address,
            existing_addresses,
            vec![
                AssetBalance::new(Asset::mock_ethereum_usdc().id.clone(), BigUint::ZERO),
                AssetBalance::new(Asset::mock_erc20().id.clone(), BigUint::from(9u32)),
            ],
        );
        assert_eq!(
            explicit_zero_changes.stale_addresses,
            vec![AssetAddress::new(Asset::mock_ethereum_usdc().id, chain_address.address.clone(), Some("5".to_string()))]
        );
        assert_eq!(
            explicit_zero_changes.latest_addresses,
            vec![AssetAddress::new(Asset::mock_erc20().id, chain_address.address.clone(), Some("9".to_string()))]
        );

        let new_zero_changes = TokenAddressChanges::from_balances(&chain_address, vec![], vec![AssetBalance::new(Asset::mock_ethereum_usdc().id.clone(), BigUint::ZERO)]);
        assert_eq!(new_zero_changes.stale_addresses, vec![]);
        assert_eq!(new_zero_changes.latest_addresses, vec![]);
    }
}
