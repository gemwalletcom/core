use std::{collections::HashMap, time::{Duration, Instant}, thread::{self, sleep}};

use blockchain::ChainProvider;
use storage::DatabaseClient;
use crate::pusher::Pusher;

pub struct Parser {
    provider: Box<dyn ChainProvider>,
    pusher: Pusher,
    database: DatabaseClient,
    options: ParserOptions,
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub timeout: u64,
}

impl Parser {
    pub fn new(
        provider: Box<dyn ChainProvider>,
        pusher: Pusher,
        database: DatabaseClient,
        options: ParserOptions
    ) -> Self {
        Self {
            provider: provider,
            pusher,
            database,
            options,
        }
    }

    pub async fn start(&mut self) {
        let chain = self.provider.get_chain();
        loop {
            let state = self.database.get_parser_state(chain).unwrap();
    
            let latest_block = self.provider.get_latest_block().await;
            match latest_block {
                Ok(latest_block) => {
                    let _ = self.database.set_parser_state_latest_block(chain, latest_block as i32);
                    if state.current_block + state.await_blocks >= state.latest_block {
                        
                        println!("parser ahead: {} current_block: {}, latest_block: {}, await_blocks: {}", chain.as_str(), state.current_block, state.latest_block, state.await_blocks);
            
                        thread::sleep(Duration::from_secs(self.options.timeout)); continue;
                    }
                 },
                Err(err) => {
                    println!("latest_block error: {:?}", err);
    
                    sleep(Duration::from_secs(self.options.timeout)); continue;
                }
            }

            let mut next_block = state.current_block + 1;
            
            loop {
                let start = Instant::now();
                let transactions = self.provider.get_transactions(next_block.into()).await;
                match transactions {
                    Ok(transactions) => {
                        let _ = self.database.set_parser_state_current_block(chain, next_block);
                        let addresses = transactions.clone().into_iter().map(|x| x.addresses() ).flatten().collect();
                        let subscriptions = self.database.get_subscriptions(chain, addresses).unwrap();
                        let mut transactions_map: HashMap<String, primitives::Transaction> = HashMap::new();
    
                        for subscription in subscriptions {
                            for transaction in transactions.clone() {
                                if transaction.addresses().contains(&subscription.address) {
                                    let device = self.database.get_device_by_id(subscription.device_id).unwrap();
                                    
                                    println!("Push: device: {}, transaction: {:?}", subscription.device_id, transaction.hash);
                                    
                                    transactions_map.insert(transaction.clone().id, transaction.clone());
    
                                    let result = self.pusher.push(device.as_primitive(), transaction.clone()).await;
                                    match result {
                                        Ok(result) => { println!("Push: result: {:?}", result); },
                                        Err(err) => { println!("Push: error: {:?}", err); }
                                    }
                                }
                            }
                        }
    
                        let insert_transactions: Vec<storage::models::Transaction> = transactions_map
                            .into_iter()
                            .map(|x| x.1)
                            .collect::<Vec<primitives::Transaction>>()
                            .into_iter().map(|x| {
                                return storage::models::Transaction::from_primitive(x);
                            }).collect();
    
                        self.database.add_transactions(insert_transactions.clone()).unwrap();

                        println!("parser block complete, chain: {}, block: {:?}, transactions: {}, insert_transactions: {}, to go blocks: {}, time elapsed: {:?}",  chain.as_str(), next_block, transactions.len(), insert_transactions.len(), state.latest_block - next_block, start.elapsed());
                    },
                    Err(err) => {
                        println!("get transactions error: {:?}", err);
                    }
                }
    
                if next_block >= state.latest_block || next_block % 100 == 0  {
                    break
                }
    
                next_block += 1;
            }
        }
    }
}
