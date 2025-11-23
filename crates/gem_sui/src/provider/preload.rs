use std::{collections::HashMap, error::Error};

use crate::{SUI_COIN_TYPE, models::SuiObject};
#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionLoad;
#[cfg(feature = "rpc")]
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{
    FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, transaction_load_metadata::SuiCoin,
};

use crate::{
    provider::preload_mapper::{GAS_BUDGET, map_transaction_data, map_transaction_rate_rates},
    rpc::client::SuiClient,
};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SuiClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (gas_coins, coins, objects) = self.get_coins_for_input_type(&input.sender_address.clone(), input.input_type.clone()).await?;
        let message_bytes = map_transaction_data(input.clone(), gas_coins.clone(), coins.clone(), objects)?;

        let fee = self.calculate_actual_fee(&message_bytes, &input.gas_price).await?;

        Ok(TransactionLoadData {
            fee,
            metadata: TransactionLoadMetadata::Sui { message_bytes },
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        Ok(map_transaction_rate_rates(gas_price))
    }
}

impl<C: Client + Clone> SuiClient<C> {
    async fn calculate_actual_fee(&self, tx_data: &str, gas_price: &GasPriceType) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let tx_data_only = tx_data.split('_').next().unwrap_or(tx_data);
        let dry_run_result = self.dry_run(tx_data_only.to_string()).await?;
        let gas_used = &dry_run_result.effects.gas_used;

        let computation_cost: BigInt = gas_used.computation_cost.clone().into();
        let storage_cost: BigInt = gas_used.storage_cost.clone().into();
        let storage_rebate: BigInt = gas_used.storage_rebate.clone().into();

        let fee = std::cmp::max(computation_cost.clone(), &computation_cost + &storage_cost - &storage_rebate);

        Ok(TransactionFee {
            fee,
            gas_price_type: gas_price.clone(),
            gas_limit: BigInt::from(GAS_BUDGET),
            options: HashMap::new(),
        })
    }
}

impl<C: Client + Clone> SuiClient<C> {
    async fn get_coins_for_input_type(
        &self,
        address: &str,
        input_type: TransactionInputType,
    ) -> Result<(Vec<SuiCoin>, Vec<SuiCoin>, Vec<SuiObject>), Box<dyn Error + Send + Sync>> {
        match input_type {
            TransactionInputType::Transfer(asset) => match asset.id.token_id {
                None => Ok((self.get_coins(address, SUI_COIN_TYPE).await?, Vec::new(), Vec::new())),
                Some(token_id) => {
                    let (gas_coins, coins) = futures::try_join!(self.get_coins(address, SUI_COIN_TYPE), self.get_coins(address, &token_id))?;
                    Ok((gas_coins, coins, Vec::new()))
                }
            },
            TransactionInputType::Stake(_, stake_type) => match stake_type {
                StakeType::Stake(_) => Ok((self.get_coins(address, SUI_COIN_TYPE).await?, Vec::new(), Vec::new())),
                StakeType::Unstake(delegation) => {
                    let (gas_coins, staked_object) =
                        futures::try_join!(self.get_coins(address, SUI_COIN_TYPE), self.get_object(delegation.base.delegation_id.clone()))?;
                    Ok((gas_coins, Vec::new(), vec![staked_object]))
                }
                _ => Err("Unsupported stake type for Sui".into()),
            },
            TransactionInputType::Swap(_, _, _) => Ok((Vec::new(), Vec::new(), Vec::new())),
            TransactionInputType::Generic(_, _, _) => Ok((Vec::new(), Vec::new(), Vec::new())),
            TransactionInputType::TransferNft(_, _) | TransactionInputType::Account(_, _) => Err("Unsupported transaction type for Sui".into()),
            _ => Err("Unsupported transaction type for Sui".into()),
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::*;
    use base64::{Engine, engine::general_purpose};
    use chain_traits::ChainTransactionLoad;
    use primitives::{Asset, Chain, FeePriority, StakeType, TransactionLoadInput};

    #[tokio::test]
    async fn test_sui_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let rates = client
            .get_transaction_fee_rates(TransactionInputType::Transfer(Asset::from_chain(Chain::Sui)))
            .await?;

        println!("Sui transaction fee rates: {:?}", rates);

        assert_eq!(rates.len(), 3);
        assert_eq!(rates[0].priority, FeePriority::Slow);
        assert_eq!(rates[1].priority, FeePriority::Normal);
        assert_eq!(rates[2].priority, FeePriority::Fast);

        Ok(())
    }

    #[tokio::test]
    async fn test_sui_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let input = TransactionPreloadInput {
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Sui)),
        };

        let _metadata = client.get_transaction_preload(input).await?;

        // match metadata {
        //     TransactionLoadMetadata::Sui { message_bytes } => {
        //         assert!(!message_bytes.is_empty());
        //         println!("Sui preload metadata: {} chars", message_bytes.len());
        //     }
        //     _ => panic!("Expected Sui metadata"),
        // }

        Ok(())
    }

    #[tokio::test]
    async fn test_sui_get_transaction_preload_unstake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();

        let delegation_id = "0x32c6c9d1de51d1df1d69687ee29c9759c06ae48e6dbb024e2cd81499b4058d51";
        let user_address = "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6";

        let delegation = primitives::Delegation::mock_with_id(delegation_id.to_string());
        let stake_type = StakeType::Unstake(delegation);

        let input = TransactionLoadInput {
            sender_address: user_address.to_string(),
            destination_address: user_address.to_string(),
            value: "1000000000".to_string(),
            input_type: TransactionInputType::Stake(Asset::from_chain(Chain::Sui), stake_type),
            gas_price: primitives::GasPriceType::regular(BigInt::from(1000)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        };

        let result = client.get_transaction_load(input).await?;

        match result.metadata {
            TransactionLoadMetadata::Sui { message_bytes } => {
                assert!(!message_bytes.is_empty());
                println!("Sui unstake transaction data: {} chars", message_bytes.len());

                assert!(message_bytes.contains('_'));
                let parts: Vec<&str> = message_bytes.split('_').collect();
                assert_eq!(parts.len(), 2);

                assert!(general_purpose::STANDARD.decode(parts[0]).is_ok());
                assert!(hex::decode(parts[1]).is_ok());
            }
            _ => panic!("Expected Sui metadata for unstake transaction"),
        }

        assert!(result.fee.fee > BigInt::from(0));
        assert_eq!(result.fee.gas_limit, BigInt::from(GAS_BUDGET));

        println!("Unstake transaction fee: {}", result.fee.fee);

        Ok(())
    }
}
