mod parser_options;

pub use parser_options::ParserOptions;

use std::{
    cmp,
    error::Error,
    time::{Duration, Instant},
};

use chain_traits::ChainTraits;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::Chain;
use settings::{Settings, service_user_agent};
use std::str::FromStr;
use storage::Database;
use streamer::{FetchBlocksPayload, QueueName, StreamProducer, TransactionsPayload};

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainTraits>,
    stream_producer: StreamProducer,
    database: Database,
    options: ParserOptions,
}

impl Parser {
    pub fn new(provider: Box<dyn ChainTraits>, stream_producer: StreamProducer, database: Database, options: ParserOptions) -> Self {
        Self {
            chain: provider.get_chain(),
            provider,
            stream_producer,
            database,
            options,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let state = self.database.client()?.parser_state().get_parser_state(self.chain.as_ref())?;
            let timeout = cmp::max(state.timeout_latest_block as u64, self.options.timeout);

            if !state.is_enabled {
                tokio::time::sleep(Duration::from_millis(timeout)).await;
                continue;
            }
            let next_current_block = state.current_block + state.await_blocks as i64;

            match self.provider.get_block_latest_number().await {
                Ok(latest_block) => {
                    let latest_block_i64 = latest_block as i64;
                    let _ = self
                        .database
                        .client()
                        .ok()
                        .and_then(|mut c| c.parser_state().set_parser_state_latest_block(self.chain.as_ref(), latest_block_i64).ok());
                    // initial start
                    if state.current_block == 0 {
                        let _ = self
                            .database
                            .client()
                            .ok()
                            .and_then(|mut c| c.parser_state().set_parser_state_current_block(self.chain.as_ref(), latest_block_i64).ok());
                    }
                    if next_current_block >= latest_block_i64 {
                        info_with_fields!(
                            "parser ahead",
                            chain = self.chain.as_ref(),
                            current_block = state.current_block,
                            latest_block = latest_block,
                            await_blocks = state.await_blocks
                        );

                        tokio::time::sleep(Duration::from_millis(timeout)).await;
                        continue;
                    }
                }
                Err(err) => {
                    error_with_fields!("parser latest_block", &*err, chain = self.chain.as_ref());

                    tokio::time::sleep(Duration::from_millis(timeout * 5)).await;
                    continue;
                }
            }

            loop {
                let start = Instant::now();
                let state = self.database.client()?.parser_state().get_parser_state(self.chain.as_ref())?;
                let start_block = state.current_block + 1;
                let end_block = cmp::min(start_block + state.parallel_blocks as i64 - 1, state.latest_block - state.await_blocks as i64);
                let next_blocks: Vec<u64> = (start_block..=end_block).map(|b| b as u64).collect();
                let remaining = state.latest_block - end_block - state.await_blocks as i64;

                if next_blocks.is_empty() {
                    break;
                }

                // queue blocks, continue parsing
                if let Some(queue_behind_blocks) = state.queue_behind_blocks
                    && remaining > queue_behind_blocks as i64
                {
                    let payload = FetchBlocksPayload::new(self.chain, next_blocks.clone());
                    self.stream_producer.publish(QueueName::FetchBlocks, &payload).await?;
                    let _ = self
                        .database
                        .client()?
                        .parser_state()
                        .set_parser_state_current_block(self.chain.as_ref(), end_block);

                    info_with_fields!(
                        "block add to queue",
                        chain = self.chain.as_ref(),
                        blocks = format!("{:?}", next_blocks),
                        remaining = remaining,
                        duration = DurationMs(start.elapsed())
                    );
                    continue;
                }

                match self.parse_blocks(next_blocks.clone()).await {
                    Ok(result) => {
                        let _ = self
                            .database
                            .client()?
                            .parser_state()
                            .set_parser_state_current_block(self.chain.as_ref(), end_block);

                        info_with_fields!(
                            "block complete",
                            chain = self.chain.as_ref(),
                            blocks = format!("{:?}", next_blocks),
                            transactions = result,
                            remaining = remaining,
                            duration = DurationMs(start.elapsed())
                        );
                    }
                    Err(err) => {
                        error_with_fields!("parser parse_block", &*err, chain = self.chain.as_ref(), blocks = format!("{:?}", next_blocks));

                        tokio::time::sleep(Duration::from_millis(timeout)).await;
                        break;
                    }
                }
                // exit loop every n blocks to update latest block
                if remaining % 50 == 0 {
                    break;
                }
                if state.timeout_between_blocks > 0 {
                    tokio::time::sleep(Duration::from_millis(state.timeout_between_blocks as u64)).await;
                    continue;
                }
            }
        }
    }

    pub async fn parse_blocks(&mut self, blocks: Vec<u64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self.provider.get_transactions_in_blocks(blocks.clone()).await?;
        if transactions.is_empty() {
            return Ok(0);
        }
        let payload = TransactionsPayload::new(self.chain, blocks.clone(), transactions.clone());
        self.stream_producer.publish(QueueName::StoreTransactions, &payload).await?;
        Ok(transactions.len())
    }
}

pub async fn run(settings: Settings, chain: Option<Chain>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);

    let chains: Vec<Chain> = if let Some(chain) = chain {
        vec![chain]
    } else {
        database
            .client()?
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
        let database = database.clone();
        let provider = settings_chain::ProviderFactory::new_from_settings_with_user_agent(chain, &settings, &service_user_agent("parser", None));
        let stream_producer = StreamProducer::new(&settings.rabbitmq.url, format!("parser_{chain}").as_str()).await.unwrap();
        let parser_options = ParserOptions {
            chain,
            timeout: settings.parser.timeout,
        };

        let parser = tokio::spawn(async move {
            parser_start(database, stream_producer, provider, parser_options).await;
        });
        parsers.push(parser);
    }

    futures::future::join_all(parsers).await;

    Ok(())
}

async fn parser_start(database: Database, stream_producer: StreamProducer, provider: Box<dyn ChainTraits>, parser_options: ParserOptions) {
    let chain = provider.get_chain();

    let mut parser = Parser::new(provider, stream_producer, database, parser_options.clone());

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
