pub mod parser;
use gem_chain_rpc::ChainBlockProvider;
pub use parser::Parser;
pub mod parser_options;
pub use parser_options::ParserOptions;
pub mod pusher;
use parser_proxy::ParserProxy;
pub use pusher::Pusher;
pub mod consumers;
pub mod parser_proxy;

use primitives::{node::NodeState, Chain};
use settings::Settings;
use std::{str::FromStr, sync::Arc, time::Duration};
use storage::{DatabaseClient, NodeStore, ParserStateStore};
use streamer::StreamProducer;
use tokio::sync::Mutex;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.last().cloned().unwrap_or_default();
    let settings = Settings::new().unwrap();
    let database = Arc::new(Mutex::new(DatabaseClient::new(&settings.postgres.url)));

    if mode == "consumers" {
        return consumers::run_consumers(settings, database.clone()).await;
    } else if mode == "consumer_transactions" {
        return consumers::run_consumer_store_transactions(settings.clone(), database.clone()).await;
    } else if mode == "consumer_blocks" {
        return consumers::run_consumer_fetch_blocks(settings, database.clone()).await;
    } else if mode == "consumer_assets" {
        tokio::spawn(consumers::run_consumer_fetch_assets(settings.clone(), database.clone()));
        tokio::spawn(consumers::run_consumer_fetch_assets_addresses_associations(settings.clone(), database.clone()));
        tokio::spawn(consumers::run_consumer_store_assets_addresses_associations(settings.clone(), database.clone()));
        std::future::pending::<()>().await;
        return Ok(());
    } else {
        return run_parser_mode(settings.clone(), database.clone()).await;
    }
}

async fn run_parser_mode(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("parser init");

    let chains: Vec<Chain> = database
        .lock()
        .await
        .get_parser_states()
        .unwrap()
        .into_iter()
        .flat_map(|x| Chain::from_str(x.chain.as_ref()))
        .collect();

    let chain_env = std::env::args().nth(1).unwrap_or_default();

    let chains = if let Ok(chain) = Chain::from_str(chain_env.as_str()) {
        vec![chain]
    } else {
        chains
    };

    let nodes = database
        .lock()
        .await
        .get_nodes()
        .unwrap()
        .into_iter()
        .map(|x| x.as_primitive())
        .filter(|x| x.node.priority > 5 && x.node.status == NodeState::Active)
        .collect::<Vec<_>>();

    println!("parser start chains: {:?}", chains);

    let mut parsers = Vec::new();
    for chain in chains {
        let settings = settings.clone();
        let proxy: ParserProxy = ParserProxy::new_from_nodes(&settings, chain, nodes.clone());
        let parser_options = ParserOptions {
            chain,
            timeout: settings.parser.timeout,
        };

        let parser = tokio::spawn(async move {
            parser_start(settings.clone(), proxy, parser_options).await;
        });
        parsers.push(parser);
    }

    futures::future::join_all(parsers).await;

    Ok(())
}

async fn parser_start(settings: Settings, proxy: ParserProxy, parser_options: ParserOptions) {
    let database_client = DatabaseClient::new(settings.postgres.url.as_str());

    let stream_producer = StreamProducer::new(&settings.rabbitmq.url).await.unwrap();

    let chain = proxy.get_chain();
    let mut parser = Parser::new(Box::new(proxy), stream_producer, database_client, parser_options.clone());

    loop {
        match parser.start().await {
            Ok(_) => {
                println!("parser start complete, chain: {}", chain)
            }
            Err(e) => {
                println!("parser start error, chain: {}, error: {:?}", chain, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(parser_options.timeout)).await;
        println!("parser restart timeout, chain: {}", chain);
    }
}
