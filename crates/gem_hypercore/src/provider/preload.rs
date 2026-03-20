use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, HyperliquidOrder, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, perpetual::PerpetualType,
};

use crate::is_spot_swap;
use crate::provider::fee_calculator::{calculate_perpetual_fee_amount, calculate_spot_fee_amount};
use crate::provider::preload_cache::HyperCoreCache;
use crate::provider::preload_mapper::get_approvals_and_credentials;
use crate::rpc::client::HyperCoreClient;

impl<C: Client> HyperCoreClient<C> {
    async fn get_order(&self, sender_address: &str) -> Result<(HyperliquidOrder, i64), Box<dyn Error + Sync + Send>> {
        let cache = HyperCoreCache::new(self.preferences.clone(), self.config.clone());
        let (agent_required, referral_required, builder_required, fee_rate, agent_address, agent_private_key) = get_approvals_and_credentials(
            &cache,
            sender_address,
            self.secure_preferences.clone(),
            self.get_extra_agents(sender_address),
            self.get_referral(sender_address),
            self.get_builder_fee(sender_address, &self.config.builder_address),
            self.get_user_fees(sender_address),
        )
        .await?;

        Ok((
            HyperliquidOrder {
                approve_agent_required: agent_required,
                approve_referral_required: referral_required,
                approve_builder_required: builder_required,
                builder_fee_bps: self.config.max_builder_fee_bps,
                agent_address,
                agent_private_key,
            },
            fee_rate,
        ))
    }
}

#[async_trait]
impl<C: Client> ChainTransactionLoad for HyperCoreClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        match &input.input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::TransferNft(_, _) | TransactionInputType::Account(_, _) | TransactionInputType::Stake(_, _) => {
                // Only signature is required
                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(BigInt::from(0)),
                    metadata: TransactionLoadMetadata::Hyperliquid { order: None },
                })
            }
            TransactionInputType::Swap(from_asset, to_asset, _) => {
                let (fee_amount, order) = if is_spot_swap(from_asset.chain(), to_asset.chain()) {
                    let (order, fee_rate) = self.get_order(&input.sender_address).await?;
                    let swap_data = input.input_type.get_swap_data().map_err(|err| err.to_string())?;
                    let fee_amount = calculate_spot_fee_amount(swap_data, from_asset, to_asset, fee_rate, self.config.max_builder_fee_bps)?;

                    (fee_amount, Some(order))
                } else {
                    (BigInt::from(0), None)
                };

                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(fee_amount),
                    metadata: TransactionLoadMetadata::Hyperliquid { order },
                })
            }
            TransactionInputType::Perpetual(_, perpetual_type) => {
                let fiat_value = match perpetual_type {
                    PerpetualType::Open(data) => data.fiat_value,
                    PerpetualType::Increase(data) => data.fiat_value,
                    PerpetualType::Reduce(reduce_data) => reduce_data.data.fiat_value,
                    PerpetualType::Close(data) => data.fiat_value,
                    PerpetualType::Modify(_) => 0.0,
                };
                let (order, fee_rate) = self.get_order(&input.sender_address).await?;
                let fee_amount = calculate_perpetual_fee_amount(fiat_value, fee_rate);

                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(fee_amount),
                    metadata: TransactionLoadMetadata::Hyperliquid { order: Some(order) },
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
mod integration_tests {
    use super::*;
    use crate::provider::testkit::create_hypercore_test_client;
    use primitives::{Asset, Chain, TransactionLoadInput};

    #[tokio::test]
    async fn test_get_transaction_load_transfer() {
        let client = create_hypercore_test_client();
        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer(Asset::from_chain(Chain::HyperCore)));

        let result = client.get_transaction_load(input).await.unwrap();

        assert_eq!(result.fee.fee, BigInt::from(0));
        let TransactionLoadMetadata::Hyperliquid { order } = result.metadata else {
            panic!("invalid metadata");
        };
        assert!(order.is_none());
    }
}
