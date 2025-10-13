use async_trait::async_trait;
use chain_traits::ChainBalances;
use num_bigint::BigUint;
use std::error::Error;

use crate::provider::balances_mapper::{map_balance_staking, map_coin_balance};
use crate::pubkey::get_token_account;
use crate::rpc::client::SolanaClient;
use gem_client::Client;
use primitives::{AssetBalance, AssetId, Chain};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for SolanaClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(&address).await?;
        Ok(map_coin_balance(&balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let (ata_addresses, token_id_map): (Vec<String>, Vec<String>) = token_ids
            .iter()
            .map(|token_id| {
                let ata = get_token_account(&address, token_id, crate::TOKEN_PROGRAM);
                (ata, token_id.clone())
            })
            .unzip();

        let balance_results = self.get_token_account_balances(&ata_addresses).await?;

        let balances = balance_results
            .iter()
            .zip(&token_id_map)
            .map(|(balance_opt, token_id)| {
                balance_opt
                    .as_ref()
                    .map(|balance| AssetBalance::new(AssetId::from_token(Chain::Solana, token_id), balance.value.amount.clone()))
                    .unwrap_or_else(|| AssetBalance::new(AssetId::from_token(Chain::Solana, token_id), BigUint::from(0u32)))
            })
            .collect();

        Ok(balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let accounts = self.get_staking_balance(&address).await?;
        Ok(map_balance_staking(accounts))
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let token_accounts_result = self.get_token_accounts_by_owner(&address, crate::TOKEN_PROGRAM).await?;
        let balances: Vec<AssetBalance> = token_accounts_result
            .value
            .into_iter()
            .filter_map(|account| {
                let token_info = &account.account.data.parsed.info;
                if let (Some(token_amount), Some(mint)) = (&token_info.token_amount, &token_info.mint)
                    && token_amount.amount > num_bigint::BigUint::from(0u32)
                {
                    let asset_id = primitives::AssetId {
                        chain: primitives::Chain::Solana,
                        token_id: Some(mint.clone()),
                    };
                    return Some(primitives::AssetBalance::new(asset_id, token_amount.amount.clone()));
                }
                None
            })
            .collect();

        Ok(balances)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_solana_test_client};
    use primitives::Chain;

    #[tokio::test]
    async fn test_solana_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Solana);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let token_ids = vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
        ];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids.clone()).await?;

        token_ids.iter().zip(balances.iter()).for_each(|(token_id, balance)| {
            println!("Token ID: {}, Balance: {}", token_id, balance.balance.available);
        });

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Solana);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let staking_balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        if let Some(balance) = staking_balance {
            assert_eq!(balance.asset_id.chain, Chain::Solana);
            assert_eq!(balance.asset_id.token_id, None);
            assert!(balance.balance.staked > num_bigint::BigUint::from(0u32));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        for asset in assets {
            assert_eq!(asset.asset_id.chain, Chain::Solana);
            assert!(asset.balance.available > num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }
}
