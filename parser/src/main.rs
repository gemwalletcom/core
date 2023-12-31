pub mod parser;
pub mod pusher;

use std::str::FromStr;

use primitives::Chain;
use settings::Settings;
use storage::DatabaseClient;

use crate::{
    parser::{Parser, ParserOptions},
    pusher::Pusher,
};

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

    println!("parser start chains: {:?}", chains);

    let mut parsers = Vec::new();
    for chain in chains {
        let settings = settings.clone();
        let parser_options = parser_options.clone();
        let parser = tokio::spawn(async move {
            parser_start(settings, parser_options, chain).await;
        });
        parsers.push(parser);
    }

    futures::future::join_all(parsers).await;
}

async fn parser_start(settings: Settings, parser_options: ParserOptions, chain: Chain) {
    let provider = settings_chain::ProviderFactory::new_provider(chain, &settings);
    let pusher = Pusher::new(
        settings.pusher.url.clone(),
        settings.postgres.url.clone(),
        settings.pusher.ios.topic.clone(),
    );
    let database_client = DatabaseClient::new(settings.postgres.url.as_str());

    let mut parser = Parser::new(provider, pusher, database_client, parser_options);
    match parser.start().await {
        Ok(_) => {
            println!("parser {} start complete", chain)
        }
        Err(e) => {
            println!("parser {} start error: {:?}", chain, e)
        }
    }
}
