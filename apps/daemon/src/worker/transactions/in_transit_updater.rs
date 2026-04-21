use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::swap::{SwapResult, SwapStatus, map_swap_result};
use primitives::{Chain, TransactionChange, TransactionMetadata, TransactionType};
use storage::models::TransactionRow;
use storage::{Database, TransactionFilter, TransactionState, TransactionUpdate, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};
use swapper::cross_chain::{self, DepositAddressMap};
use swapper::swapper::GemSwapper;

use crate::client::SwapVaultAddressClient;

#[derive(Clone, Copy)]
pub struct InTransitConfig {
    pub timeout: Duration,
    pub query_limit: i64,
}

pub struct InTransitUpdater {
    database: Database,
    config: InTransitConfig,
    swapper: Arc<GemSwapper>,
    stream_producer: StreamProducer,
    vault_client: SwapVaultAddressClient,
}

impl InTransitUpdater {
    pub fn new(database: Database, config: InTransitConfig, swapper: Arc<GemSwapper>, stream_producer: StreamProducer, vault_client: SwapVaultAddressClient) -> Self {
        Self {
            database,
            config,
            swapper,
            stream_producer,
            vault_client,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self
            .database
            .transactions()?
            .get_transactions_by_filter(vec![TransactionFilter::States(vec![TransactionState::InTransit])], self.config.query_limit)?;

        if transactions.is_empty() {
            return Ok(0);
        }

        let vault_addresses = self.vault_client.get_deposit_address_map().await?;
        let cutoff = (Utc::now() - self.config.timeout).naive_utc();
        let mut updated = 0;

        for transaction in &transactions {
            if self.process_transaction(transaction, cutoff, &vault_addresses).await? {
                updated += 1;
            }
        }

        Ok(updated)
    }

    async fn process_transaction(&self, row: &TransactionRow, cutoff: NaiveDateTime, vault_addresses: &DepositAddressMap) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let chain = row.chain();
        let transaction = row.as_primitive(row.get_addresses());
        let elapsed = DurationMs((Utc::now().naive_utc() - row.created_at).to_std().unwrap_or_default());

        let provider = cross_chain::swap_provider_with_vault_addresses(&transaction, vault_addresses);
        let provider_name = provider.map(|p| p.as_ref().to_string()).unwrap_or_default();
        let result = match provider {
            Some(provider) => match self.swapper.get_swap_result(chain, provider, &row.hash).await {
                Ok(r) => r,
                Err(err) => {
                    error_with_fields!(
                        "in_transit check failed",
                        &err as &dyn Error,
                        chain = chain.as_ref(),
                        hash = row.hash,
                        provider = provider_name,
                        elapsed = elapsed
                    );
                    if row.created_at < cutoff {
                        info_with_fields!("in_transit timed out", chain = chain.as_ref(), hash = row.hash, provider = provider_name, elapsed = elapsed);
                        self.save_and_publish(chain, row, &TransactionState::Failed, None).await?;
                        return Ok(true);
                    }
                    return Ok(false);
                }
            },
            None => SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
            },
        };
        let update = map_swap_result(&result);
        let state: TransactionState = if !update.state.is_terminal() && row.created_at < cutoff {
            TransactionState::Failed
        } else if update.state.is_terminal() {
            update.state.into()
        } else {
            info_with_fields!("in_transit pending", chain = chain.as_ref(), hash = row.hash, provider = provider_name, elapsed = elapsed);
            return Ok(false);
        };

        info_with_fields!("in_transit confirmed", chain = chain.as_ref(), hash = row.hash, state = state.as_ref(), elapsed = elapsed);

        let metadata = update.changes.into_iter().find_map(|change| match change {
            TransactionChange::Metadata(TransactionMetadata::Swap(m)) => serde_json::to_value(&m).ok(),
            _ => None,
        });
        self.save_and_publish(chain, row, &state, metadata).await?;
        Ok(true)
    }

    async fn save_and_publish(
        &self,
        chain: Chain,
        row: &TransactionRow,
        state: &TransactionState,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut updates = vec![TransactionUpdate::State(state.clone()), TransactionUpdate::Kind(TransactionType::Swap.into())];
        if let Some(ref json) = metadata {
            updates.push(TransactionUpdate::Metadata(json.clone()));
        }
        self.database.transactions()?.update_transaction(chain.as_ref(), &row.hash, updates)?;

        let transaction = row.as_primitive(row.get_addresses()).with_swap_state(state.clone().into(), metadata.clone());
        self.stream_producer
            .publish_transactions(TransactionsPayload::new_with_notify(chain, vec![], vec![transaction]))
            .await?;
        Ok(())
    }
}
