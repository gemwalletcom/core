mod parser_options;
mod parser_state;

pub use parser_options::ParserOptions;
use parser_state::ParserStateService;

use std::{
    cmp,
    error::Error,
    time::{Duration, Instant},
};

use cacher::CacherClient;
use chain_traits::ChainTraits;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::Chain;
use settings::Settings;
use std::str::FromStr;
use storage::Database;
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

use crate::shutdown::{self, ShutdownReceiver};

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainTraits>,
    stream_producer: StreamProducer,
    state_service: ParserStateService,
    options: ParserOptions,
    shutdown_rx: ShutdownReceiver,
}

impl Parser {
    pub fn new(
        provider: Box<dyn ChainTraits>,
        stream_producer: StreamProducer,
        database: Database,
        cacher: CacherClient,
        options: ParserOptions,
        shutdown_rx: ShutdownReceiver,
    ) -> Self {
        let chain = provider.get_chain();
        let state_service = ParserStateService::new(chain, database, cacher);
        Self {
            chain,
            provider,
            stream_producer,
            state_service,
            options,
            shutdown_rx,
        }
    }

    fn is_shutdown(&self) -> bool {
        *self.shutdown_rx.borrow()
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.state_service.init().await?;

        let mut last_persist = Instant::now();

        loop {
            if self.is_shutdown() {
                info_with_fields!("shutdown requested", chain = self.chain.as_ref());
                break;
            }

            if last_persist.elapsed() >= self.options.persist_interval {
                self.state_service.persist_state().await;
                last_persist = Instant::now();
            }

            let state = self.state_service.get_state()?;
            let timeout = cmp::max(Duration::from_millis(state.timeout_latest_block as u64), self.options.timeout);

            if !state.is_enabled {
                if shutdown::sleep_or_shutdown(timeout, &self.shutdown_rx).await {
                    break;
                }
                continue;
            }

            let current_block = self.state_service.get_current_block().await;
            let next_current_block = current_block + state.await_blocks as i64;

            match self.provider.get_block_latest_number().await {
                Ok(latest_block) => {
                    let latest_block_i64 = latest_block as i64;
                    let _ = self.state_service.set_latest_block(latest_block_i64).await;

                    if current_block == 0 {
                        let _ = self.state_service.set_current_block(latest_block_i64).await;
                    }

                    if next_current_block >= latest_block_i64 {
                        info_with_fields!(
                            "parser ahead",
                            chain = self.chain.as_ref(),
                            current_block = current_block,
                            latest_block = latest_block,
                            await_blocks = state.await_blocks
                        );
                        if shutdown::sleep_or_shutdown(timeout, &self.shutdown_rx).await {
                            break;
                        }
                        continue;
                    }
                }
                Err(err) => {
                    error_with_fields!("parser latest_block", &*err, chain = self.chain.as_ref());
                    if shutdown::sleep_or_shutdown(timeout * 5, &self.shutdown_rx).await {
                        break;
                    }
                    continue;
                }
            }

            loop {
                if self.is_shutdown() {
                    break;
                }

                let start = Instant::now();
                let current_block = self.state_service.get_current_block().await;
                let latest_block = self.state_service.get_latest_block().await;

                let start_block = current_block + 1;
                let end_block = cmp::min(start_block + state.parallel_blocks as i64 - 1, latest_block - state.await_blocks as i64);
                let next_blocks: Vec<u64> = (start_block..=end_block).map(|b| b as u64).collect();
                let remaining = latest_block - end_block - state.await_blocks as i64;

                if next_blocks.is_empty() {
                    break;
                }

                if let Some(queue_behind_blocks) = state.queue_behind_blocks
                    && remaining > queue_behind_blocks as i64
                {
                    self.stream_producer.publish_blocks(self.chain, &next_blocks).await?;
                    let _ = self.state_service.set_current_block(end_block).await;

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
                        let _ = self.state_service.set_current_block(end_block).await;

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
                        shutdown::sleep_or_shutdown(timeout, &self.shutdown_rx).await;
                        break;
                    }
                }

                if remaining % self.options.catchup_reload_interval == 0 {
                    break;
                }
                if state.timeout_between_blocks > 0 && shutdown::sleep_or_shutdown(Duration::from_millis(state.timeout_between_blocks as u64), &self.shutdown_rx).await {
                    break;
                }
            }
        }

        self.state_service.persist_state().await;
        info_with_fields!("parser stopped", chain = self.chain.as_ref());

        Ok(())
    }

    async fn parse_blocks(&self, blocks: Vec<u64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transactions = self.provider.get_transactions_in_blocks(blocks.clone()).await?;
        if transactions.is_empty() {
            return Ok(0);
        }
        let count = transactions.len();
        let payload = TransactionsPayload::new(self.chain, blocks, transactions);
        self.stream_producer.publish_transactions(payload).await?;
        Ok(count)
    }
}

pub async fn run(settings: Settings, chain: Option<Chain>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let cacher = CacherClient::new(&settings.redis.url).await;

    let config = storage::ConfigCacher::new(database.clone());
    let catchup_reload_interval = config.get_i64(primitives::ConfigKey::ParserCatchupReloadInterval)?;
    let persist_interval = config.get_duration(primitives::ConfigKey::ParserPersistInterval)?;

    let chains: Vec<Chain> = if let Some(chain) = chain {
        vec![chain]
    } else {
        database
            .parser_state()?
            .get_parser_states()?
            .into_iter()
            .flat_map(|x| Chain::from_str(x.chain.as_ref()))
            .collect()
    };

    info_with_fields!("parser init", chains = format!("{:?}", chains));

    let (shutdown_tx, shutdown_rx) = shutdown::channel();
    let shutdown_timeout = settings.parser.shutdown.timeout;

    let signal_handle = shutdown::spawn_signal_handler(shutdown_tx);

    let mut handles = Vec::new();

    for chain in chains {
        let database = database.clone();
        let cacher = cacher.clone();
        let shutdown_rx = shutdown_rx.clone();
        let settings = settings.clone();

        let provider = settings_chain::ProviderFactory::new_from_settings_with_user_agent(chain, &settings, &settings::service_user_agent("parser", None));

        let stream_producer = StreamProducer::new(&settings.rabbitmq.url, format!("parser_{chain}").as_str()).await?;

        let options = ParserOptions {
            timeout: settings.parser.timeout,
            catchup_reload_interval,
            persist_interval,
        };

        handles.push(tokio::spawn(async move {
            run_parser(database, cacher, stream_producer, provider, options, shutdown_rx).await;
        }));
    }

    signal_handle.await.ok();
    shutdown::wait_with_timeout(handles, shutdown_timeout).await;

    info_with_fields!("all parsers stopped", status = "ok");
    Ok(())
}

async fn run_parser(
    database: Database,
    cacher: CacherClient,
    stream_producer: StreamProducer,
    provider: Box<dyn ChainTraits>,
    options: ParserOptions,
    shutdown_rx: ShutdownReceiver,
) {
    let chain = provider.get_chain();
    let timeout = options.timeout;

    let parser = Parser::new(provider, stream_producer, database, cacher, options, shutdown_rx.clone());

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        if let Err(e) = parser.start().await {
            error_with_fields!("parser error", &*e, chain = chain.as_ref());

            if shutdown::sleep_or_shutdown(timeout, &shutdown_rx).await {
                break;
            }
        }
    }
}
