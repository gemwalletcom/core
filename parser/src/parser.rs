use std::{collections::HashMap, time::{Duration, Instant}, cmp, error::Error};
use tokio::time::sleep;

use blockchain::ChainProvider;
use primitives::Chain;
use storage::DatabaseClient;
use crate::pusher::Pusher;

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainProvider>,
    pusher: Pusher,
    database: DatabaseClient,
    options: ParserOptions,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub timeout: u64,
}

#[derive(Debug, Clone)]
pub struct ParserBlocksResult {
    pub transactions: usize,
    pub insert_transactions: usize,
}


impl Parser {
    pub fn new(
        provider: Box<dyn ChainProvider>,
        pusher: Pusher,
        database: DatabaseClient,
        options: ParserOptions
    ) -> Self {
        Self {
            chain: provider.get_chain(),
            provider,
            pusher,
            database,
            options,
        }
    }

    pub async fn start(&mut self) {    
        loop {
            let state = self.database.get_parser_state(self.chain).unwrap();
            
            if !state.is_enabled {
                sleep(Duration::from_millis(self.options.timeout)).await; continue;
            }

            let next_current_block = state.current_block + state.await_blocks;

            // skip fetching latest block if parsed is not up to date
            if next_current_block >= state.latest_block  {
                match self.provider.get_latest_block().await {
                    Ok(latest_block) => {
                        let _ = self.database.set_parser_state_latest_block(self.chain, latest_block as i32);
                        if next_current_block >= latest_block as i32 {
                            
                            println!("parser ahead: {} current_block: {}, latest_block: {}, await_blocks: {}", self.chain.as_str(), state.current_block, latest_block, state.await_blocks);
                
                            sleep(Duration::from_millis(self.options.timeout)).await; continue;
                        }
                     },
                    Err(err) => {
                        println!("parser latest_block chain: {}, error: {:?}", self.chain.as_str(), err);
        
                        sleep(Duration::from_millis(self.options.timeout)).await; continue;
                    }
                }
            }

            loop {
                let state = self.database.get_parser_state(self.chain).unwrap();
                let start = Instant::now();
                let start_block =  state.current_block + 1;
                let end_block = cmp::min(start_block + state.parallel_blocks - 1, state.latest_block - state.await_blocks);
                let next_blocks = (start_block..=end_block).collect::<Vec<_>>();

                if next_blocks.len() == 0 {
                    break
                }
                
                match self.parse_blocks(next_blocks.clone()).await {
                    Ok(result) => {
                        let _ = self.database.set_parser_state_current_block(self.chain, end_block.into());
                        
                        println!("parser block complete: {}, blocks: {:?} transactions: {} of {}, to go blocks: {}, in: {:?}",  self.chain.as_str(), next_blocks, result.transactions, result.insert_transactions, state.latest_block - end_block - state.await_blocks, start.elapsed());
                     },
                    Err(err) => { 
                        println!("parser parse_block chain: blocks: {}, {:?}, error: {:?}", self.chain.as_str(), next_blocks, err);

                        sleep(Duration::from_millis(self.options.timeout)).await; continue;
                    }
                }
                if state.timeout_between_blocks > 0 {
                    sleep(Duration::from_millis(state.timeout_between_blocks.try_into().unwrap())).await; continue;
                }
            }
        }
    }

    pub async fn parse_blocks(&mut self, blocks: Vec<i32>) -> Result<ParserBlocksResult, Box<dyn Error + Send + Sync>> {
        let transactions = futures::future::try_join_all(
            blocks.iter().map(|block| self.provider.get_transactions(block.clone() as i64))
        ).await?;
        let transactions = transactions.into_iter().flatten().collect::<Vec<primitives::Transaction>>();

        let addresses = transactions.clone().into_iter().map(|x| x.addresses() ).flatten().collect();
        let subscriptions = self.database.get_subscriptions(self.chain, addresses).unwrap();
        let mut transactions_map: HashMap<String, primitives::Transaction> = HashMap::new();

        for subscription in subscriptions {
            for transaction in transactions.clone() {
                if transaction.addresses().contains(&subscription.address) {
                    let device = self.database.get_device_by_id(subscription.device_id).unwrap();
                    
                    println!("Push: device: {}, chain: {}, transaction: {:?}", subscription.device_id, self.chain.as_str(), transaction.hash);
                    
                    transactions_map.insert(transaction.clone().id, transaction.clone());

                    match self.pusher.push(device.as_primitive(), transaction.clone(), subscription.as_primitive() ).await {
                        Ok(result) => { println!("Push: result: {:?}", result); },
                        Err(err) => { println!("Push: error: {:?}", err); }
                    }
                }
            }
        }
        
        match self.store_transactions(transactions_map.clone()).await {
            Ok(_) => { },
            Err(err) => { println!("transaction insert: error: {:?}", err); }
        }

        return Ok(ParserBlocksResult{
            transactions: transactions.len(), 
            insert_transactions: transactions_map.len()
        });
    }

    pub async fn store_transactions(&mut self, transactions_map: HashMap<String, primitives::Transaction>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let insert_transactions: Vec<storage::models::Transaction> = transactions_map
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<primitives::Transaction>>()
        .into_iter().map(|x| {
            return storage::models::Transaction::from_primitive(x);
        }).collect();
        let result =  self.database.add_transactions(insert_transactions.clone())?;
        Ok(result)
    }
}
