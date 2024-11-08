use std::{
    cmp,
    collections::HashMap,
    error::Error,
    time::{Duration, Instant},
};

use crate::{ParserOptions, Pusher};
use gem_chain_rpc::ChainBlockProvider;
use primitives::Chain;
use storage::DatabaseClient;

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainBlockProvider>,
    pusher: Pusher,
    database: DatabaseClient,
    options: ParserOptions,
}

#[derive(Debug, Clone)]
pub struct ParserBlocksResult {
    pub transactions: usize,
    pub insert_transactions: usize,
}

impl Parser {
    pub fn new(provider: Box<dyn ChainBlockProvider>, pusher: Pusher, database: DatabaseClient, options: ParserOptions) -> Self {
        Self {
            chain: provider.get_chain(),
            provider,
            pusher,
            database,
            options,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let state = self.database.get_parser_state(self.chain)?;

            if !state.is_enabled {
                tokio::time::sleep(Duration::from_millis(self.options.timeout)).await;
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

                        tokio::time::sleep(Duration::from_millis(self.options.timeout)).await;
                        continue;
                    }
                }
                Err(err) => {
                    println!("parser latest_block chain: {}, error: {:?}", self.chain.as_ref(), err);

                    tokio::time::sleep(Duration::from_millis(self.options.timeout * 5)).await;
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
                            "parser block complete: {}, blocks: {:?} transactions: {} of {}, to go blocks: {}, in: {:?}",
                            self.chain.as_ref(),
                            next_blocks,
                            result.transactions,
                            result.insert_transactions,
                            to_go_blocks,
                            start.elapsed()
                        );
                    }
                    Err(err) => {
                        println!("parser parse_block chain: blocks: {}, {:?}, error: {:?}", self.chain.as_ref(), next_blocks, err);

                        tokio::time::sleep(Duration::from_millis(self.options.timeout)).await;
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

    async fn fetch_blocks(&mut self, blocks: Vec<i32>) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let mut retry_attempts_count = 0;
        loop {
            let results = futures::future::try_join_all(blocks.iter().map(|block| self.provider.get_transactions(*block as i64))).await;
            match results {
                Ok(transactions) => return Ok(transactions.into_iter().flatten().collect::<Vec<primitives::Transaction>>()),
                Err(err) => {
                    if retry_attempts_count >= self.options.retry {
                        return Err(err);
                    }
                    retry_attempts_count += 1;

                    tokio::time::sleep(Duration::from_millis(retry_attempts_count * self.options.timeout * 2)).await;
                }
            }
        }
    }

    pub async fn parse_blocks(&mut self, blocks: Vec<i32>) -> Result<ParserBlocksResult, Box<dyn Error + Send + Sync>> {
        let transactions = self.fetch_blocks(blocks.clone()).await?;
        let addresses = transactions.clone().into_iter().flat_map(|x| x.addresses()).collect();
        let subscriptions = self.database.get_subscriptions(self.chain, addresses)?;
        let mut transactions_map: HashMap<String, primitives::Transaction> = HashMap::new();

        // Debugging only, insert all transactions
        // for transaction in transactions.clone().into_iter() {
        //     transactions_map.insert(transaction.clone().id, transaction.clone());
        // }

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.address) {
                    let device = self.database.get_device_by_id(subscription.device_id)?;

                    println!(
                        "push: device: {}, chain: {}, transaction: {:?}",
                        subscription.device_id,
                        self.chain.as_ref(),
                        transaction.hash
                    );

                    transactions_map.insert(transaction.clone().id, transaction.clone());

                    let transaction = transaction.finalize(vec![subscription.address.clone()]).clone();

                    if self.options.is_transaction_outdated(transaction.asset_id.chain, transaction.created_at) {
                        println!("outdated transaction: {}, created_at: {}", transaction.id, transaction.created_at);
                        continue;
                    }

                    match self.pusher.push(device.as_primitive(), transaction, subscription.as_primitive()).await {
                        Ok(result) => {
                            println!("push: result: {:?}", result);
                        }
                        Err(err) => {
                            println!("push: error: {:?}", err);
                        }
                    }
                }
            }
        }

        match self.store_transactions(transactions_map.clone()).await {
            Ok(_) => {}
            Err(err) => {
                println!("transaction insert: chain: {}, blocks: {:?}, error: {:?}", self.chain.as_ref(), blocks, err);
            }
        }

        Ok(ParserBlocksResult {
            transactions: transactions.len(),
            insert_transactions: transactions_map.len(),
        })
    }

    pub async fn store_transactions(&mut self, transactions_map: HashMap<String, primitives::Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let primitive_transactions = transactions_map
            .clone()
            .into_iter()
            .map(|x| x.1)
            .collect::<Vec<primitives::Transaction>>()
            .into_iter()
            .filter(|x| {
                let asset_ids = x.asset_ids();
                if let Ok(assets) = self.database.get_assets(asset_ids.clone()) {
                    assets.len() == asset_ids.len()
                } else {
                    false
                }
            })
            .collect::<Vec<primitives::Transaction>>();

        let transaction_chunks = primitive_transactions.chunks(300);

        for chunk in transaction_chunks {
            let transactions = chunk
                .to_vec()
                .clone()
                .into_iter()
                .map(storage::models::Transaction::from_primitive)
                .collect::<Vec<storage::models::Transaction>>();

            let transaction_addresses = chunk
                .to_vec()
                .clone()
                .into_iter()
                .flat_map(|transaction| storage::models::TransactionAddresses::from_primitive(transaction))
                .collect::<Vec<storage::models::TransactionAddresses>>();

            if transactions.is_empty() || transaction_addresses.is_empty() {
                return Ok(primitive_transactions.len());
            }

            self.database.add_transactions(transactions.clone(), transaction_addresses.clone())?;
        }

        Ok(primitive_transactions.len())
    }
}
