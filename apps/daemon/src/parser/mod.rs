mod parser_options;
mod parser_state;
mod plan;

pub use parser_options::ParserOptions;
use parser_state::ParserStateService;

use std::{
    error::Error,
    time::{Duration, Instant},
};

use cacher::{CacheKey, CacherClient};
use chain_traits::ChainTraits;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::{Chain, ParserError, ParserStatus};
use settings::Settings;
use std::str::FromStr;
use streamer::{StreamProducer, StreamProducerConfig, StreamProducerQueue, TransactionsPayload};

use crate::shutdown::{self, ShutdownReceiver};
use plan::{BlockPlan, BlockPlanKind, plan_next_block, should_reload_catchup, timeout_for_state};
use storage::{Database, models::ParserStateRow};

pub struct Parser {
    chain: Chain,
    provider: Box<dyn ChainTraits>,
    stream_producer: StreamProducer,
    state_service: ParserStateService,
    cacher: CacherClient,
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
        let state_service = ParserStateService::new(chain, database, cacher.clone());
        Self {
            chain,
            provider,
            stream_producer,
            state_service,
            cacher,
            options,
            shutdown_rx,
        }
    }

    fn is_shutdown(&self) -> bool {
        *self.shutdown_rx.borrow()
    }

    async fn sleep_or_shutdown(&self, duration: Duration) -> bool {
        shutdown::sleep_or_shutdown(duration, &self.shutdown_rx).await
    }

    async fn wait_if_disabled(&self, state: &ParserStateRow, timeout: Duration) -> bool {
        if state.is_enabled {
            true
        } else {
            self.sleep_or_shutdown(timeout).await;
            false
        }
    }

    async fn persist_if_due(&self, last_persist: &mut Instant) {
        if last_persist.elapsed() >= self.options.persist_interval {
            self.state_service.persist_state().await;
            *last_persist = Instant::now();
        }
    }

    async fn refresh_latest_block(&self, state: &ParserStateRow, timeout: Duration) -> Result<bool, Box<dyn Error + Send + Sync>> {
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
                    if self.sleep_or_shutdown(timeout).await {
                        return Ok(false);
                    }
                    return Ok(false);
                }
            }
            Err(err) => {
                error_with_fields!("parser latest_block", &*err, chain = self.chain.as_ref());
                if self.sleep_or_shutdown(timeout * 5).await {
                    return Ok(false);
                }
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn execute_plan(&self, plan: BlockPlan, state: &ParserStateRow, timeout: Duration) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let start = Instant::now();
        let blocks_desc = format!("{:?}", plan.range.blocks);

        match plan.kind {
            BlockPlanKind::Enqueue => {
                self.stream_producer.publish_blocks(self.chain, &plan.range.blocks).await?;
                let _ = self.state_service.set_current_block(plan.range.end_block).await;

                info_with_fields!(
                    "block add to queue",
                    chain = self.chain.as_ref(),
                    blocks = blocks_desc,
                    remaining = plan.range.remaining,
                    duration = DurationMs(start.elapsed())
                );
                return Ok(true);
            }
            BlockPlanKind::Parse => {}
        }

        match self.parse_blocks(plan.range.blocks).await {
            Ok(result) => {
                let _ = self.state_service.set_current_block(plan.range.end_block).await;

                info_with_fields!(
                    "block complete",
                    chain = self.chain.as_ref(),
                    blocks = blocks_desc,
                    transactions = result,
                    remaining = plan.range.remaining,
                    duration = DurationMs(start.elapsed())
                );
            }
            Err(err) => {
                error_with_fields!("parser parse_block", &*err, chain = self.chain.as_ref(), blocks = blocks_desc);
                self.sleep_or_shutdown(timeout).await;
                return Ok(false);
            }
        }

        if should_reload_catchup(plan.range.remaining, self.options.catchup_reload_interval) {
            return Ok(false);
        }
        if state.timeout_between_blocks > 0
            && self.sleep_or_shutdown(Duration::from_millis(state.timeout_between_blocks as u64)).await
        {
            return Ok(false);
        }

        Ok(true)
    }

    async fn process_blocks(&self, state: &ParserStateRow, timeout: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            if self.is_shutdown() {
                break;
            }

            let current_block = self.state_service.get_current_block().await;
            let latest_block = self.state_service.get_latest_block().await;

            let Some(plan) = plan_next_block(state, current_block, latest_block) else {
                break;
            };

            if !self.execute_plan(plan, state, timeout).await? {
                break;
            }
        }

        Ok(())
    }

    async fn report_error(&self, error: &str) {
        let cache_key = CacheKey::ParserStatus(self.chain.as_ref());
        let key = cache_key.key();
        let mut status = self.cacher.get_value::<ParserStatus>(&key).await.unwrap_or_default();
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        let truncated = if error.len() > 200 { error[..200].to_string() } else { error.to_string() };

        if let Some(entry) = status.errors.iter_mut().find(|e| e.message == truncated) {
            entry.count += 1;
            entry.timestamp = timestamp;
        } else {
            status.errors.push(ParserError {
                message: truncated,
                count: 1,
                timestamp,
            });
        }

        if let Err(e) = self.cacher.set_cached(cache_key, &status).await {
            info_with_fields!("parser status report failed", chain = self.chain.as_ref(), error = format!("{:?}", e));
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.state_service.init().await?;

        let mut last_persist = Instant::now();

        loop {
            if self.is_shutdown() {
                info_with_fields!("shutdown requested", chain = self.chain.as_ref());
                break;
            }

            self.persist_if_due(&mut last_persist).await;
            let state = self.state_service.get_state()?;
            let timeout = timeout_for_state(&state, self.options.timeout);

            if !self.wait_if_disabled(&state, timeout).await {
                continue;
            }

            if !self.refresh_latest_block(&state, timeout).await? {
                continue;
            }

            self.process_blocks(&state, timeout).await?;
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

        let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
        let stream_producer = StreamProducer::new(&rabbitmq_config, format!("parser_{chain}").as_str()).await?;

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
    info_with_fields!("waiting for parser shutdown", tasks = handles.len());
    let _ = shutdown::wait_with_timeout(handles, shutdown_timeout).await;

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
            let error_msg = format!("{:?}", e);
            error_with_fields!("parser error", &*e, chain = chain.as_ref());
            parser.report_error(&error_msg).await;

            if shutdown::sleep_or_shutdown(timeout, &shutdown_rx).await {
                break;
            }
        }
    }
}
