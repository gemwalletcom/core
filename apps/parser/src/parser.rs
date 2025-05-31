use std::{
    cmp,
    error::Error,
    time::{Duration, Instant},
};

use crate::ParserOptions;
use gem_chain_rpc::ChainBlockProvider;
use primitives::{Chain, Transaction, TransactionType};
use storage::DatabaseClient;
use streamer::{QueueName, StreamProducer};

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainBlockProvider>,
    stream_producer: StreamProducer,
    database: DatabaseClient,
    options: ParserOptions,
}

#[derive(Debug, Clone)]
pub struct ParserBlocksResult {
    pub transactions: usize,
}

impl Parser {
    pub fn new(provider: Box<dyn ChainBlockProvider>, stream_producer: StreamProducer, database: DatabaseClient, options: ParserOptions) -> Self {
        Self {
            chain: provider.get_chain(),
            provider,
            stream_producer,
            database,
            options,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let state = self.database.get_parser_state(self.chain)?;
            let timeout = cmp::max(state.timeout_latest_block as u64, self.options.timeout);

            if !state.is_enabled {
                tokio::time::sleep(Duration::from_millis(timeout)).await;
                continue;
            }
            let next_current_block = state.current_block + state.await_blocks;

            match self.provider.get_latest_block().await {
                Ok(latest_block) => {
                    let _ = self.database.set_parser_state_latest_block(self.chain, latest_block as i32);
                    // initial start
                    if state.current_block == 0 {
                        let _ = self.database.set_parser_state_current_block(self.chain, latest_block as i32);
                    }
                    if next_current_block >= latest_block as i32 {
                        println!(
                            "parser ahead: {} current_block: {}, latest_block: {}, await_blocks: {}",
                            self.chain.as_ref(),
                            state.current_block,
                            latest_block,
                            state.await_blocks
                        );

                        tokio::time::sleep(Duration::from_millis(timeout)).await;
                        continue;
                    }
                }
                Err(err) => {
                    println!("parser latest_block chain: {}, error: {:?}", self.chain.as_ref(), err);

                    tokio::time::sleep(Duration::from_millis(timeout * 5)).await;
                    continue;
                }
            }

            loop {
                let state = self.database.get_parser_state(self.chain)?;
                let start = Instant::now();
                let start_block = state.current_block + 1;
                let end_block = cmp::min(start_block + state.parallel_blocks - 1, state.latest_block - state.await_blocks);
                let next_blocks = (start_block..=end_block).collect::<Vec<_>>();
                let to_go_blocks = state.latest_block - end_block - state.await_blocks;

                if next_blocks.is_empty() {
                    break;
                }

                match self.parse_blocks(next_blocks.clone()).await {
                    Ok(result) => {
                        let _ = self.database.set_parser_state_current_block(self.chain, end_block);

                        println!(
                            "parser block complete: {}, blocks: {:?} transactions: {} to go blocks: {} in: {:?}",
                            self.chain.as_ref(),
                            next_blocks,
                            result,
                            to_go_blocks,
                            start.elapsed()
                        );
                    }
                    Err(err) => {
                        println!("parser parse_block chain: blocks: {}, {:?}, error: {:?}", self.chain.as_ref(), next_blocks, err);

                        tokio::time::sleep(Duration::from_millis(timeout)).await;
                        break;
                    }
                }
                // exit loop every n blocks to update latest block
                if to_go_blocks % 100 == 0 {
                    break;
                }
                if state.timeout_between_blocks > 0 {
                    tokio::time::sleep(Duration::from_millis(state.timeout_between_blocks as u64)).await;
                    continue;
                }
            }
        }
    }

    async fn fetch_blocks(&mut self, blocks: Vec<i32>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let results = futures::future::try_join_all(blocks.iter().map(|block| self.provider.get_transactions(*block as i64))).await;
        match results {
            Ok(transactions) => Ok(transactions.into_iter().flatten().collect::<Vec<primitives::Transaction>>()),
            Err(err) => Err(err),
        }
    }
    pub async fn parse_blocks(&mut self, blocks: Vec<i32>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self
            .fetch_blocks(blocks.clone())
            .await?
            .into_iter()
            .filter(|x| filter_transaction(&x.value, &x.transaction_type, self.options.minimum_transfer_amount()))
            .collect::<Vec<Transaction>>();

        if transactions.is_empty() {
            return Ok(0);
        }

        let payload = streamer::TransactionsPayload {
            chain: self.chain,
            blocks: blocks.clone(),
            transactions: transactions.clone(),
        };

        self.stream_producer.publish(QueueName::Transactions, &payload).await?;

        Ok(transactions.len())
    }
}

fn filter_transaction(value: &str, transaction_type: &TransactionType, minimum_transfer_amount: u64) -> bool {
    if *transaction_type == TransactionType::Transfer {
        if let Ok(value) = value.parse::<u64>() {
            return value >= minimum_transfer_amount;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_transaction() {
        let test_cases = vec![
            ("1", TransactionType::Transfer, 0, true),
            ("500", TransactionType::Transfer, 1000, false),
            ("1000", TransactionType::Transfer, 1000, true),
            ("1500", TransactionType::Transfer, 1000, true),
            ("invalid", TransactionType::Transfer, 1000, true),
        ];

        for (transaction_value, transaction_type, minimum_transfer_amount, expected) in test_cases {
            assert_eq!(filter_transaction(transaction_value, &transaction_type, minimum_transfer_amount), expected);
        }
    }
}
