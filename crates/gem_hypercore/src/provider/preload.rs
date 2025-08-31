use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    perpetual::PerpetualType, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionPreloadInput,
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
            TransactionInputType::Perpetual(_, perpetual_type) => {
                let cache = HyperCoreCache::new(self.preferences.clone(), self.config.clone());

                let fiat_value = match perpetual_type {
                    PerpetualType::Open(data) => data.fiat_value,
                    PerpetualType::Close(data) => data.fiat_value,
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

                let metadata = TransactionLoadMetadata::Hyperliquid {
                    approve_agent_required: agent_required,
                    approve_referral_required: referral_required,
                    approve_builder_required: builder_required,
                    builder_fee_bps: self.config.max_builder_fee_bps,
                    agent_address,
                    agent_private_key,
                };

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
