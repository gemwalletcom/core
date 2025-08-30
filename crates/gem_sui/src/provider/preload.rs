use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionLoad;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SuiClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::default(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(gas_price))])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::*;
    use chain_traits::ChainTransactionLoad;
    use primitives::{Asset, Chain};

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
}
