mod parser;
mod parser_options;

pub use parser::Parser;
pub use parser_options::ParserOptions;

use chain_traits::ChainTraits;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::Chain;
use settings::{Settings, service_user_agent};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use storage::DatabaseClient;
use streamer::StreamProducer;
use tokio::sync::Mutex;

pub async fn run(settings: Settings, chain: Option<Chain>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database = Arc::new(Mutex::new(DatabaseClient::new(&settings.postgres.url)));

    let chains: Vec<Chain> = if let Some(chain) = chain {
        vec![chain]
    } else {
        database
            .lock()
            .await
            .parser_state()
            .get_parser_states()
            .unwrap()
            .into_iter()
            .flat_map(|x| Chain::from_str(x.chain.as_ref()))
            .collect()
    };

    info_with_fields!("parser init", chains = format!("{:?}", chains));

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

async fn parser_start(settings: Settings, provider: Box<dyn ChainTraits>, parser_options: ParserOptions) {
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
