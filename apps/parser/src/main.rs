pub mod parser;
pub use parser::Parser;
pub mod parser_options;
pub use parser_options::ParserOptions;
pub mod pusher;
pub use pusher::Pusher;
pub mod consumers;

use gem_tracing::{SentryConfig, SentryTracing, error_with_fields, info_with_fields};
use primitives::Chain;
use settings::{Settings, service_user_agent};
use std::{str::FromStr, sync::Arc, time::Duration};
use storage::DatabaseClient;
use streamer::StreamProducer;
use tokio::sync::Mutex;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.last().cloned().unwrap_or_default();
    let settings = Settings::new().unwrap();

    let sentry_config = settings.sentry.as_ref().map(|s| SentryConfig {
        dsn: s.dsn.clone(),
        sample_rate: s.sample_rate,
    });
    let _tracing = SentryTracing::init(sentry_config.as_ref(), "parser");

    let database = Arc::new(Mutex::new(DatabaseClient::new(&settings.postgres.url)));

    if mode == "consumers" {
        return consumers::run_consumers(settings, database.clone()).await;
    } else if mode == "consumer_fetch_transactions" {
        return consumers::run_consumer_fetch_transactions(settings.clone(), database.clone()).await;
    } else if mode == "consumer_store_transactions" {
        return consumers::run_consumer_store_transactions(settings.clone(), database.clone()).await;
    } else if mode == "consumer_fetch_blocks" {
        return consumers::run_consumer_fetch_blocks(settings.clone()).await;
    } else if mode == "consumer_fetch_assets" {
        return consumers::run_consumer_fetch_assets(settings.clone(), database.clone()).await;
    } else if mode == "consumer_fetch_token_addresses_mappings" {
        return consumers::run_consumer_fetch_token_addresses_mappings(settings.clone(), database.clone()).await;
    } else if mode == "consumer_fetch_coin_addresses_mappings" {
        return consumers::run_consumer_fetch_coin_addresses_mappings(settings.clone(), database.clone()).await;
    } else if mode == "consumer_store_assets_mappings" {
        return consumers::run_consumer_store_assets_mappings(settings.clone(), database.clone()).await;
    } else if mode == "consumer_fetch_nft_assets_mappings" {
        return consumers::run_consumer_fetch_nft_assets_mappings(settings.clone(), database.clone()).await;
    } else if mode.starts_with("consumer_") {
        panic!("Unknown consumer mode: {}", mode);
    } else {
        return run_parser_mode(settings.clone(), database.clone()).await;
    }
}

async fn run_parser_mode(settings: Settings, database: Arc<Mutex<DatabaseClient>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("parser init", mode = "parser");

    let chains: Vec<Chain> = database
        .lock()
        .await
        .parser_state()
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

    info_with_fields!("parser start chains", chains = format!("{:?}", chains));

    let mut parsers = Vec::new();
    for chain in chains {
        let settings = settings.clone();
        let provider = settings_chain::ProviderFactory::new_from_settings_with_user_agent(chain, &settings, &service_user_agent("parser", None));
        let parser_options = ParserOptions {
            chain,
            timeout: settings.parser.timeout,
        };

        let parser = tokio::spawn(async move {
            parser_start(settings.clone(), provider, parser_options).await;
        });
        parsers.push(parser);
    }

    futures::future::join_all(parsers).await;

    Ok(())
}

async fn parser_start(settings: Settings, provider: Box<dyn chain_traits::ChainTraits>, parser_options: ParserOptions) {
    let database_client = DatabaseClient::new(settings.postgres.url.as_str());
    let chain = provider.get_chain();
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, format!("parser_{chain}").as_str()).await.unwrap();

    let mut parser = Parser::new(provider, stream_producer, database_client, parser_options.clone());

    loop {
        match parser.start().await {
            Ok(_) => {
                info_with_fields!("parser start complete", chain = chain.as_ref())
            }
            Err(e) => {
                error_with_fields!("parser start error", &*e, chain = chain.as_ref());
            }
        }
        tokio::time::sleep(Duration::from_millis(parser_options.timeout)).await;
        info_with_fields!("parser restart timeout", chain = chain.as_ref());
    }
}
