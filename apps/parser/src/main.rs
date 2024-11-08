pub mod parser;
pub use parser::Parser;
pub mod parser_options;
pub use parser_options::ParserOptions;
pub mod pusher;
use parser_proxy::{ParserProxy, ParserProxyUrlConfig};
pub use pusher::Pusher;
pub mod parser_proxy;

use api_connector::PusherClient;
use primitives::Chain;
use settings::Settings;
use std::{collections::HashMap, str::FromStr, time::Duration};
use storage::DatabaseClient;

#[tokio::main]
pub async fn main() {
    println!("parser init");

    let settings: Settings = Settings::new().unwrap();

    let mut database = DatabaseClient::new(&settings.postgres.url.clone());
    let chains: Vec<Chain> = database
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
    let parser_options = ParserOptions {
        timeout: settings.parser.timeout,
        retry: settings.parser.retry,
    };

    // create node client
    let nodes = database.get_nodes().unwrap();

    let mut nodes_map: HashMap<String, Vec<String>> = HashMap::new();
    nodes.into_iter().for_each(|node| {
        nodes_map.entry(node.chain.clone()).or_default().push(node.url);
    });

    println!("parser start chains: {:?}", chains);

    let mut parsers = Vec::new();
    for chain in chains {
        let settings = settings.clone();
        let parser_options = parser_options.clone();
        let node_urls = nodes_map.clone().get(chain.as_ref()).cloned().unwrap_or_default();

        let parser = tokio::spawn(async move {
            parser_start(settings, parser_options, chain, node_urls).await;
        });
        parsers.push(parser);
    }

    futures::future::join_all(parsers).await;
}

async fn parser_start(settings: Settings, parser_options: ParserOptions, chain: Chain, node_urls: Vec<String>) {
    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let pusher = Pusher::new(settings.postgres.url.clone(), pusher_client);
    let database_client = DatabaseClient::new(settings.postgres.url.as_str());

    let url = settings_chain::ProviderFactory::url(chain, &settings);
    let node_urls = if node_urls.is_empty() { vec![url.to_string()] } else { node_urls };
    let config = ParserProxyUrlConfig { urls: node_urls };
    let proxy = ParserProxy::new(chain, config);

    let mut parser = Parser::new(Box::new(proxy), pusher, database_client, parser_options.clone());
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
