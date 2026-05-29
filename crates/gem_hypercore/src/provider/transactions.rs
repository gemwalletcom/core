use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    models::{
        order::UserFill,
        spot::SpotMeta,
        transaction_id::{HyperCoreActionId, HyperCoreTransactionId},
    },
    provider::{
        transaction_state_mapper,
        transactions_mapper::{map_user_fill_by_hash, map_user_fill_by_oid, map_user_fills},
    },
    rpc::client::HyperCoreClient,
};

const TRANSACTION_ID_PREFIX: &str = "hypercore_";

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let start_time = request.from_timestamp.map(|ts| ts as i64 * 1000).unwrap_or(0);
        let fills = self.get_user_fills_by_time(&request.address, start_time).await?;
        let spot_meta = load_spot_meta_if_needed(self, &fills).await?;
        let transactions = map_user_fills(&request.address, fills, spot_meta.as_ref());

        match request.asset_id {
            Some(asset_id) => Ok(transactions.into_iter().filter(|tx| tx.asset_ids().contains(&asset_id)).collect()),
            None => Ok(transactions),
        }
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let hash = hash.strip_prefix(TRANSACTION_ID_PREFIX).unwrap_or(&hash);

        if hash.starts_with("0x") {
            return self.get_transaction_by_tx_hash(hash).await;
        }

        match HyperCoreTransactionId::parse(hash) {
            Some(HyperCoreTransactionId::Order(oid)) => self.get_transaction_by_order_id(oid, hash).await,
            Some(HyperCoreTransactionId::Action(action_id)) => self.get_transaction_by_action_id(hash, action_id).await,
            None => Ok(None),
        }
    }
}

impl<C: Client> HyperCoreClient<C> {
    async fn get_transaction_by_tx_hash(&self, hash: &str) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let response = self.get_transaction_details(hash).await?;
        let sender = response.tx.user.to_lowercase();
        self.cache_transaction_sender(hash, &sender)?;

        self.map_user_fills_with_spot_meta(
            &sender,
            response.tx.time.saturating_sub(transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS as i64),
            |fills, spot_meta| map_user_fill_by_hash(&sender, fills, hash, spot_meta),
        )
        .await
    }

    async fn get_transaction_by_order_id(&self, oid: u64, id: &str) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(sender) = self.get_cached_transaction_sender(id)? else {
            return Ok(None);
        };

        self.map_user_fills_with_spot_meta(&sender, 0, |fills, spot_meta| map_user_fill_by_oid(&sender, fills, oid, spot_meta))
            .await
    }

    async fn get_transaction_by_action_id(&self, id: &str, action_id: HyperCoreActionId) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        match action_id {
            HyperCoreActionId::Order(nonce) => self.get_transaction_by_order_action_id(id, nonce).await,
            HyperCoreActionId::Nonce(_) | HyperCoreActionId::CDeposit { .. } | HyperCoreActionId::CWithdraw { .. } | HyperCoreActionId::TokenDelegate { .. } => {
                self.get_transaction_by_ledger_action_id(id, action_id).await
            }
        }
    }

    async fn get_transaction_by_order_action_id(&self, id: &str, nonce: u64) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(sender) = self.get_cached_transaction_sender(id)? else {
            return Ok(None);
        };

        self.map_user_fills_with_spot_meta(
            &sender,
            nonce.saturating_sub(transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS) as i64,
            |fills, spot_meta| {
                let oid = transaction_state_mapper::order_action_fill(&fills, nonce)?.oid;
                map_user_fill_by_oid(&sender, fills, oid, spot_meta)
            },
        )
        .await
    }

    async fn get_transaction_by_ledger_action_id(&self, id: &str, action_id: HyperCoreActionId) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(sender) = self.get_cached_transaction_sender(id)? else {
            return Ok(None);
        };

        let updates = self
            .get_ledger_updates(&sender, action_id.nonce().saturating_sub(transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS) as i64)
            .await?;
        let Some(hash) = transaction_state_mapper::ledger_action_hash(&updates, &action_id) else {
            return Ok(None);
        };
        self.get_transaction_by_tx_hash(&hash).await
    }

    async fn map_user_fills_with_spot_meta<F>(&self, sender: &str, start_time: i64, map: F) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>>
    where
        F: FnOnce(Vec<UserFill>, Option<&SpotMeta>) -> Option<Transaction>,
    {
        let fills = self.get_user_fills_by_time(sender, start_time).await?;
        let spot_meta = load_spot_meta_if_needed(self, &fills).await?;
        Ok(map(fills, spot_meta.as_ref()))
    }
}

async fn load_spot_meta_if_needed<C: Client>(client: &HyperCoreClient<C>, fills: &[UserFill]) -> Result<Option<SpotMeta>, Box<dyn Error + Sync + Send>> {
    if fills.iter().any(|fill| fill.coin.starts_with('@')) {
        return Ok(Some(client.get_spot_meta().await?));
    }
    Ok(None)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_TRANSACTION_ID, TEST_TRANSACTION_ORDER_ID, create_hypercore_test_client};

    #[tokio::test]
    async fn test_hypercore_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let transaction = client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await?.unwrap();
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);

        let sender = client.get_cached_transaction_sender(TEST_TRANSACTION_ID)?.unwrap();
        let order_id = HyperCoreTransactionId::Order(TEST_TRANSACTION_ORDER_ID.parse()?).to_string();
        client.cache_transaction_sender(&order_id, &sender)?;

        let transaction = client.get_transaction_by_hash(format!("hypercore_{order_id}")).await?.unwrap();
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);

        Ok(())
    }
}
