use std::{thread::{sleep, self}, time::Duration};

pub mod pusher;

use blockchain::ChainProvider;
use primitives::{chain::Chain, Transaction};
use settings::Settings;
use storage::DatabaseClient;

use crate::pusher::Pusher;

#[tokio::main]
pub async fn main() {
    println!("Hello, parser!");

    let settings: Settings = Settings::new().unwrap();
    let mut database_client: DatabaseClient = DatabaseClient::new(&settings.postgres.url);
    let bnbchain_client = blockchain::BNBChainClient::new(
        settings.chains.binance.url,
        settings.chains.binance.api
    );
    let mut pusher = Pusher::new(
        settings.pusher.url,
        settings.postgres.url,
        settings.pusher.ios.topic,
    );

    // let providers: Vec<Box<dyn ChainProvider>> = vec![
    //     Box::new(blockchain::BNBChainClient::new(
    //         settings.chains.binance.url,
    //         settings.chains.binance.api
    //     )),
    //     Box::new(blockchain::BNBChainClient::new(
    //         settings.chains.binance.url,
    //         settings.chains.binance.api
    //     )),
    // ];

    // for provider in providers {
    //     tokio::spawn(async move {

    //         println!("launch provider: {:?}", provider.get_chain());

    //         loop {
    //             let latest_block: i32 = provider.get_latest_block().await.unwrap();
    //             println!("latest_block: {:?}", latest_block);

    //             //thread::sleep(Duration::from_secs(2))
    //         }
    //     });
    // }

    loop {
        let state = database_client.get_parser_state(Chain::Binance).unwrap();

        let latest_block = bnbchain_client.get_latest_block().await;
        match latest_block {
            Ok(latest_block) => {
                let _ = database_client.set_parser_state_latest_block(Chain::Binance, latest_block);
                if state.current_block + state.await_blocks >= state.latest_block {
                    
                    println!("parser ahead. current_block: {}, latest_block: {}, await_blocks: {}", state.current_block, state.latest_block, state.await_blocks);
        
                    thread::sleep(Duration::from_secs(settings.pusher.timeout)); continue;
                }
             },
            Err(err) => {
                println!("latest_block error: {:?}", err);

                sleep(Duration::from_secs(settings.pusher.timeout)); continue;
            }
        }
        
        println!("current_block: {}, latest_block: {}", state.current_block, state.latest_block);
 
        let mut next_block = state.current_block + 1;

        loop {
            println!("next_block: {:?}, to go: {}", next_block, state.latest_block - next_block);

            let transactions = bnbchain_client.get_transactions(next_block).await;
            match transactions {
                Ok(transactions) => {
                    let _ = database_client.set_parser_state_current_block(Chain::Binance, next_block);

                    let addresses = transactions.clone().into_iter().map(|x| x.addresses() ).flatten().collect();
                    let subscriptions = database_client.get_subscriptions(Chain::Binance, addresses).unwrap();
                    let mut store_transactions: Vec<Transaction> = vec![];

                    for subscription in subscriptions {
                        for transaction in transactions.clone() {
                            if transaction.addresses().contains(&subscription.address) {
                                let device = database_client.get_device_by_id(subscription.device_id).unwrap();
                                println!("Push: device: {}, transaction: {:?}", subscription.device_id, transaction.hash);

                                store_transactions.push(transaction.clone());

                                let result = pusher.push(device.as_primitive(), transaction.clone()).await;
                                match result {
                                    Ok(result) => { println!("Push: result: {:?}", result); },
                                    Err(err) => { println!("Push: error: {:?}", err); }
                                }
                            }
                        }
                    }

                    let db_transactions = store_transactions.into_iter().map(|transaction| {
                        storage::models::Transaction::from_primitive(transaction)
                    }).collect();

                    database_client.add_transactions(db_transactions).unwrap();
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