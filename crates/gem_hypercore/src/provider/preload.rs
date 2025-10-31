use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, HyperliquidOrder, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionPreloadInput, perpetual::PerpetualType,
};

use crate::provider::preload_cache::HyperCoreCache;
use crate::provider::preload_mapper::{calculate_fee_amount, get_approvals_and_credentials};
use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for HyperCoreClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        match &input.input_type {
            TransactionInputType::Transfer(_)
            | TransactionInputType::TransferNft(_, _)
            | TransactionInputType::Account(_, _)
            | TransactionInputType::Swap(_, _, _)
            | TransactionInputType::Stake(_, _) => {
                // Only signature is required
                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(BigInt::from(0)),
                    metadata: TransactionLoadMetadata::Hyperliquid { order: None },
                })
            }
            TransactionInputType::Perpetual(_, perpetual_type) => {
                let cache = HyperCoreCache::new(self.preferences.clone(), self.config.clone());

                let fiat_value = match perpetual_type {
                    PerpetualType::Open(data) => data.fiat_value,
                    PerpetualType::Increase(data) => data.fiat_value,
                    PerpetualType::Reduce(reduce_data) => reduce_data.data.fiat_value,
                    PerpetualType::Close(data) => data.fiat_value,
                    PerpetualType::Modify(_) => 0.0,
                };

                let (agent_required, referral_required, builder_required, fee_rate, agent_address, agent_private_key) = get_approvals_and_credentials(
                    &cache,
                    &input.sender_address,
                    self.secure_preferences.clone(),
                    self.get_extra_agents(&input.sender_address),
                    self.get_referral(&input.sender_address),
                    self.get_builder_fee(&input.sender_address, &self.config.builder_address),
                    self.get_user_fees(&input.sender_address),
                )
                .await?;

                let fee_amount = calculate_fee_amount(fiat_value, fee_rate);

                let order = Some(HyperliquidOrder {
                    approve_agent_required: agent_required,
                    approve_referral_required: referral_required,
                    approve_builder_required: builder_required,
                    builder_fee_bps: self.config.max_builder_fee_bps,
                    agent_address,
                    agent_private_key,
                });

                let metadata = TransactionLoadMetadata::Hyperliquid { order };

                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(fee_amount),
                    metadata,
                })
            }
            _ => Err("Unsupported input type".to_string().into()),
        }
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod tests {
    use super::*;
    use crate::provider::testkit::create_hypercore_test_client;
    use primitives::{Asset, Chain, TransactionLoadInput};

    #[tokio::test]
    async fn test_get_transaction_load_transfer() {
        let client = create_hypercore_test_client();
        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer(Asset::from_chain(Chain::HyperCore)));

        let result = client.get_transaction_load(input).await.unwrap();

        assert_eq!(result.fee.fee, BigInt::from(0));
        assert!(matches!(result.metadata, TransactionLoadMetadata::Hyperliquid { order: None }));
    }
}
