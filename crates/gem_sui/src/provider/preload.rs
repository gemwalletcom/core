use std::{collections::HashMap, error::Error};

use crate::SUI_COIN_TYPE;
#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionLoad;
#[cfg(feature = "rpc")]
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{
    transaction_load_metadata::SuiCoin, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionPreloadInput,
};

use crate::{
    provider::preload_mapper::{calculate_fee_rates, map_transaction_data, GAS_BUDGET},
    rpc::client::SuiClient,
};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SuiClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (gas_coins, coins) = self.get_coins_for_input_type(&input.sender_address.clone(), input.input_type.clone()).await?;
        let message_bytes = map_transaction_data(input.clone(), gas_coins.clone(), coins.clone())?;

        let fee = self.calculate_actual_fee(&message_bytes, &input.gas_price).await?;

        Ok(TransactionLoadData {
            fee,
            metadata: TransactionLoadMetadata::Sui { message_bytes },
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        Ok(calculate_fee_rates(gas_price))
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
    ) -> Result<(Vec<SuiCoin>, Vec<SuiCoin>), Box<dyn Error + Send + Sync>> {
        match input_type {
            TransactionInputType::Transfer(asset) => match asset.id.token_id {
                None => Ok((self.get_coins(address, SUI_COIN_TYPE).await?, Vec::new())),
                Some(token_id) => Ok(futures::try_join!(self.get_coins(address, SUI_COIN_TYPE), self.get_coins(address, &token_id))?),
            },
            TransactionInputType::Stake(..) => Ok((self.get_coins(address, SUI_COIN_TYPE).await?, Vec::new())),
            TransactionInputType::Swap(_, _, _) => Ok((Vec::new(), Vec::new())),
            _ => Err("Unsupported transaction type for Sui".into()),
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::*;
    use chain_traits::ChainTransactionLoad;
    use primitives::{Asset, Chain, FeePriority};

    #[tokio::test]
    async fn test_sui_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let rates = client
            .get_transaction_fee_rates(TransactionInputType::Transfer(Asset::from_chain(Chain::Sui)))
            .await?;

        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].priority, FeePriority::Normal);
        println!("Sui transaction fee rates: {:?}", rates);

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

        let metadata = client.get_transaction_preload(input).await?;

        match metadata {
            TransactionLoadMetadata::Sui { message_bytes } => {
                assert!(!message_bytes.is_empty());
                println!("Sui preload metadata: {} chars", message_bytes.len());
            }
            _ => panic!("Expected Sui metadata"),
        }

        Ok(())
    }
}
