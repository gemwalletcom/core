use std::{
    cmp,
    error::Error,
    time::{Duration, Instant},
};

use crate::ParserOptions;
use gem_chain_rpc::ChainBlockProvider;
use primitives::Chain;
use storage::DatabaseClient;
use streamer::{FetchBlocksPayload, QueueName, StreamProducer, TransactionsPayload};

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
                let start = Instant::now();
                let state = self.database.get_parser_state(self.chain)?;
                let start_block = state.current_block + 1;
                let end_block = cmp::min(start_block + state.parallel_blocks - 1, state.latest_block - state.await_blocks);
                let next_blocks = (start_block..=end_block).map(|x| x as i64).collect::<Vec<_>>();
                let to_go_blocks = state.latest_block - end_block - state.await_blocks;

                if next_blocks.is_empty() {
                    break;
                }

                // queue blocks, continue parsing
                if let Some(queue_behind_blocks) = state.queue_behind_blocks {
                    if to_go_blocks > queue_behind_blocks {
                        let payload = FetchBlocksPayload::new(self.chain, next_blocks.clone());
                        self.stream_producer.publish(QueueName::FetchBlocks, &payload).await?;
                        let _ = self.database.set_parser_state_current_block(self.chain, end_block);

                        println!(
                            "parser block add to queue: {}, blocks: {:?} to go blocks: {} in: {:?}",
                            self.chain.as_ref(),
                            next_blocks,
                            to_go_blocks,
                            start.elapsed()
                        );
                        continue;
                    }
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

    pub async fn parse_blocks(&mut self, blocks: Vec<i64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self.provider.get_transactions_in_blocks(blocks.clone()).await?;
        if transactions.is_empty() {
            return Ok(0);
        }
        let payload = TransactionsPayload::new(self.chain, blocks.clone(), transactions.clone());
        self.stream_producer.publish(QueueName::Transactions, &payload).await?;
        Ok(transactions.len())
    }
}
