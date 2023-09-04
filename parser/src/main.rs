use std::{thread::{sleep, self}, time::Duration};

use blockchain::ChainProvider;
use primitives::chain::Chain;
use settings::Settings;
use storage::database::DatabaseClient;

#[tokio::main]
pub async fn main() {
    println!("Hello, parser!");

    let settings: Settings = Settings::new().unwrap();
    let mut database_client: DatabaseClient = DatabaseClient::new(&settings.postgres.url);
    let bnbchain_client = blockchain::BNBChainClient::new(
        settings.chains.binance.url.to_string(),
        "https://api.binance.org".to_string()
    );

    loop {
        let state = database_client.get_parser_state(Chain::Binance).unwrap();

        let latest_block = bnbchain_client.get_latest_block().await;
        match latest_block {
            Ok(latest_block) => {
                let _ = database_client.set_parser_state_latest_block(Chain::Binance, latest_block);
                if state.current_block >= state.latest_block {
                    println!("parser ahead. current_block: {}, latest_block: {}", state.current_block, state.latest_block);
        
                    thread::sleep(Duration::from_secs(2)); continue;
                }
             },
            Err(err) => {
                println!("latest_block error: {:?}", err);

                sleep(Duration::from_secs(2)); continue;
            }
        }
        
        println!("current_block: {}, latest_block: {}", state.current_block, state.latest_block);
 
        let mut next_block = state.current_block + 1;

        loop {
            println!("next_block: {:?}, to go: {}", next_block, state.latest_block - next_block);

            let transactions = bnbchain_client.get_transactions(next_block).await;
            match transactions {
                Ok(_) => {
                    let _ = database_client.set_parser_state_current_block(Chain::Binance, next_block);

                    //println!("transactions: {:?}", transactions);
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