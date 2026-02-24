use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::swap::SwapResult;
use primitives::{Chain, TransactionSwapMetadata, TransactionType};
use storage::models::TransactionRow;
use storage::{Database, TransactionFilter, TransactionState, TransactionUpdate, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};
use swapper::SwapperProvider;
use swapper::cross_chain;
use swapper::swapper::GemSwapper;

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
}

impl InTransitUpdater {
    pub fn new(database: Database, config: InTransitConfig, swapper: Arc<GemSwapper>, stream_producer: StreamProducer) -> Self {
        Self {
            database,
            config,
            swapper,
            stream_producer,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self
            .database
            .transactions()?
            .get_transactions_by_filter(vec![TransactionFilter::State(TransactionState::InTransit)], self.config.query_limit)?;

        info_with_fields!("in_transit transactions", count = transactions.len());

        if transactions.is_empty() {
            return Ok(0);
        }

        let cutoff = (Utc::now() - self.config.timeout).naive_utc();
        let mut updated = 0;

        for transaction in &transactions {
            let chain = transaction.chain();
            let metadata = parse_metadata(&transaction.metadata);

            let Some(provider) = resolve_provider(metadata.as_ref(), &chain, transaction.to_address.as_deref(), transaction.memo.as_deref()) else {
                continue;
            };

            let result = match self.swapper.get_swap_result(chain, provider, &transaction.hash).await {
                Ok(r) => r,
                Err(err) => {
                    error_with_fields!("in_transit check failed", &err as &dyn Error, chain = chain.as_ref(), hash = transaction.hash);
                    continue;
                }
            };

            let Some((state, merged)) = resolve_status(&result, metadata, transaction.created_at, cutoff) else {
                info_with_fields!("in_transit pending", chain = chain.as_ref(), hash = transaction.hash);
                continue;
            };

            info_with_fields!("in_transit updated", chain = chain.as_ref(), hash = transaction.hash, state = state.as_ref());

            let metadata_json = merged.and_then(|m| serde_json::to_value(m).ok());
            self.update_transaction(chain.as_ref(), &transaction.hash, &state, metadata_json.as_ref())?;
            self.publish_transaction(chain, transaction, state.into(), metadata_json).await?;
            updated += 1;
        }

        Ok(updated)
    }

    fn update_transaction(&self, chain: &str, hash: &str, state: &TransactionState, metadata: Option<&serde_json::Value>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut updates = vec![TransactionUpdate::State(state.clone()), TransactionUpdate::Kind(TransactionType::Swap.into())];
        if let Some(json) = metadata {
            updates.push(TransactionUpdate::Metadata(json.clone()));
        }
        self.database.transactions()?.update_transaction(chain, hash, updates)?;
        Ok(())
    }

    async fn publish_transaction(
        &self,
        chain: Chain,
        row: &TransactionRow,
        state: primitives::TransactionState,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut transaction = row.as_primitive(row.get_addresses());
        transaction.state = state;
        transaction.transaction_type = TransactionType::Swap;
        if let Some(m) = metadata {
            transaction.metadata = Some(m);
        }
        self.stream_producer
            .publish_transactions(TransactionsPayload::new(chain, vec![0], vec![transaction]))
            .await?;
        Ok(())
    }
}

fn parse_metadata(value: &Option<serde_json::Value>) -> Option<TransactionSwapMetadata> {
    value.as_ref().and_then(|m| serde_json::from_value(m.clone()).ok())
}

fn resolve_provider(metadata: Option<&TransactionSwapMetadata>, chain: &Chain, to_address: Option<&str>, memo: Option<&str>) -> Option<SwapperProvider> {
    metadata
        .and_then(|m| m.provider.as_deref())
        .and_then(|p| SwapperProvider::from_str(p).ok())
        .or_else(|| to_address.and_then(|addr| cross_chain::swap_provider(chain, addr, memo)))
}

fn resolve_status(
    result: &SwapResult,
    existing: Option<TransactionSwapMetadata>,
    created_at: NaiveDateTime,
    cutoff: NaiveDateTime,
) -> Option<(TransactionState, Option<TransactionSwapMetadata>)> {
    let metadata = result.metadata.clone().or(existing);
    match result.status.transaction_state() {
        Some(state) => Some((state.into(), metadata)),
        None if created_at < cutoff => Some((TransactionState::Failed, metadata)),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::TransactionState as PrimitiveTransactionState;
    use primitives::swap::SwapStatus;

    fn swap_result(status: SwapStatus, metadata: Option<TransactionSwapMetadata>) -> SwapResult {
        SwapResult { status, metadata }
    }

    fn swap_metadata(provider: &str, from_value: &str, to_value: &str) -> TransactionSwapMetadata {
        TransactionSwapMetadata {
            from_asset: "bitcoin".into(),
            from_value: from_value.to_string(),
            to_asset: "ethereum".into(),
            to_value: to_value.to_string(),
            provider: Some(provider.to_string()),
        }
    }

    #[test]
    fn test_resolve_status_completed() {
        let now = Utc::now().naive_utc();
        let (state, _) = resolve_status(&swap_result(SwapStatus::Completed, None), None, now, now).unwrap();
        assert_eq!(*state, PrimitiveTransactionState::Confirmed);
    }

    #[test]
    fn test_resolve_status_failed() {
        let now = Utc::now().naive_utc();
        let (state, _) = resolve_status(&swap_result(SwapStatus::Failed, None), None, now, now).unwrap();
        assert_eq!(*state, PrimitiveTransactionState::Failed);
    }

    #[test]
    fn test_resolve_status_pending_within_timeout() {
        let now = Utc::now().naive_utc();
        let cutoff = (Utc::now() - Duration::from_secs(3600)).naive_utc();
        assert!(resolve_status(&swap_result(SwapStatus::Pending, None), None, now, cutoff).is_none());
    }

    #[test]
    fn test_resolve_status_pending_past_timeout() {
        let cutoff = Utc::now().naive_utc();
        let created_at = (Utc::now() - Duration::from_secs(7200)).naive_utc();
        let (state, _) = resolve_status(&swap_result(SwapStatus::Pending, None), None, created_at, cutoff).unwrap();
        assert_eq!(*state, PrimitiveTransactionState::Failed);
    }

    #[test]
    fn test_resolve_status_metadata_fallback() {
        let now = Utc::now().naive_utc();
        let existing = swap_metadata("thorchain", "50000", "2500");
        let (_, resolved) = resolve_status(&swap_result(SwapStatus::Completed, None), Some(existing), now, now).unwrap();
        assert_eq!(resolved.unwrap().from_value, "50000");
    }

    #[test]
    fn test_resolve_status_result_metadata_takes_precedence() {
        let now = Utc::now().naive_utc();
        let existing = swap_metadata("thorchain", "50000", "2500");
        let from_result = swap_metadata("thorchain", "100000", "5000");
        let (_, resolved) = resolve_status(&swap_result(SwapStatus::Completed, Some(from_result)), Some(existing), now, now).unwrap();
        assert_eq!(resolved.unwrap().from_value, "100000");
    }

    #[test]
    fn test_resolve_provider_from_metadata() {
        let metadata = swap_metadata("thorchain", "1000", "500");
        assert_eq!(resolve_provider(Some(&metadata), &Chain::Ethereum, None, None), Some(SwapperProvider::Thorchain));
    }

    #[test]
    fn test_resolve_provider_none() {
        assert!(resolve_provider(None, &Chain::Ethereum, None, None).is_none());
    }

    #[test]
    fn test_resolve_provider_no_provider_field() {
        let metadata = TransactionSwapMetadata {
            from_asset: "ethereum".into(),
            from_value: "1000".to_string(),
            to_asset: "bitcoin".into(),
            to_value: "500".to_string(),
            provider: None,
        };
        assert!(resolve_provider(Some(&metadata), &Chain::Ethereum, None, None).is_none());
    }
}
