mod consumers;
mod health;
mod metrics;
mod model;
mod parser;
mod pusher;
mod reporters;
mod setup;
mod shutdown;
mod worker;

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::model::{ConsumerService, DaemonService, WorkerService};
use crate::reporters::consumer::ConsumerReporter;
use crate::reporters::job::JobReporter;
use crate::shutdown::ShutdownReceiver;
use crate::worker::context::WorkerContext;
use crate::worker::job_schedule::CacherJobTracker;
use crate::worker::runtime::WorkerRuntime;
use cacher::CacherClient;
use gem_tracing::{error_with_fields, info_with_fields};
use job_runner::{JobHandle, JobSchedule};
use std::sync::atomic::{AtomicBool, Ordering};
use streamer::ConsumerStatusReporter;

#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let service_arg = args.iter().skip(1).map(|s| s.as_str()).collect::<Vec<_>>().join(" ");

    let service = DaemonService::from_str(&service_arg).unwrap_or_else(|e| {
        panic!(
            "{}\nUsage examples: \n daemon parser \n daemon parser ethereum \n daemon worker alerter \n daemon consumer fetch_transactions",
            e
        );
    });

    let settings = settings::Settings::new().unwrap();

    info_with_fields!("daemon start", service = service.name());

    match service {
        DaemonService::Setup => {
            let _ = setup::run_setup(settings).await;
        }
        DaemonService::SetupDev => {
            let _ = setup::run_setup_dev(settings).await;
        }
        DaemonService::Worker(service) => {
            let services = match service {
                Some(worker) => vec![worker],
                None => WorkerService::all(),
            };
            run_worker_services(settings, &services).await;
        }
        DaemonService::Parser(chain) => {
            let parser_metrics = Arc::new(metrics::parser::ParserMetrics::new());
            let health_state = health::spawn_server(parser_metrics.clone());
            parser::run(settings, chain, health_state, parser_metrics).await.expect("Parser failed");
        }
        DaemonService::Consumer(service) => {
            let services = match service {
                Some(consumer) => vec![consumer],
                None => ConsumerService::all(),
            };
            run_consumer_services(settings, &services).await.expect("Consumer failed");
        }
    }
}

async fn run_worker_services(settings: settings::Settings, services: &[WorkerService]) {
    if services.is_empty() {
        info_with_fields!("no worker services requested", status = "ok");
        return;
    }

    let settings = Arc::new(settings);
    let (shutdown_tx, shutdown_rx) = shutdown::channel();
    let shutdown_timeout = settings.daemon.shutdown.timeout;

    let scheduler_cacher = CacherClient::new(&settings.metrics.redis.url).await;
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);

    let service_name = services.first().map(|s| s.as_ref()).unwrap_or("worker");
    let job_metrics = Arc::new(metrics::job::JobMetrics::new(service_name));
    let price_metrics = Arc::new(metrics::price::PriceMetrics::new());
    let composite = Arc::new(metrics::Metrics::new(vec![job_metrics.clone(), price_metrics.clone()]));
    let health_state = health::spawn_server(composite);

    let signal_handle = shutdown::spawn_signal_handler(shutdown_tx);

    let worker_jobs: Vec<_> = futures::future::join_all(services.iter().map(|service| {
        let svc = *service;
        let tracker = Arc::new(CacherJobTracker::new(scheduler_cacher.clone(), service.as_ref()));
        let reporter = Arc::new(JobReporter::new(job_metrics.clone()));
        let schedule: Arc<dyn JobSchedule> = tracker;
        let runtime = WorkerRuntime::new(reporter, schedule);
        let context = WorkerContext::new(settings.clone(), database.clone(), runtime);
        let shutdown_rx = shutdown_rx.clone();
        let price_metrics = price_metrics.clone();
        async move {
            match svc.run_jobs(context, shutdown_rx, price_metrics).await {
                Ok(handles) => Some((svc, handles)),
                Err(err) => {
                    error_with_fields!("worker init failed", &*err, worker = svc.as_ref());
                    None
                }
            }
        }
    }))
    .await
    .into_iter()
    .flatten()
    .collect();

    let job_count: usize = worker_jobs.iter().map(|(_, jobs)| jobs.len()).sum();
    health_state.set_ready();
    info_with_fields!("workers ready", workers = worker_jobs.len(), jobs = job_count);

    signal_handle.await.ok();

    if worker_jobs.is_empty() {
        info_with_fields!("no workers started", status = "ok");
        return;
    }

    let status_tracks = collect_status_tracks(&worker_jobs);
    log_pending_workers(&status_tracks, "waiting for worker shutdown");

    let handles_only: Vec<_> = worker_jobs.into_iter().flat_map(|(_, jobs)| jobs.into_iter().map(JobHandle::into_handle)).collect();
    let completed = shutdown::wait_with_timeout(handles_only, shutdown_timeout).await;

    if !completed {
        log_pending_workers(&status_tracks, "force-stopping unfinished jobs");
    }

    info_with_fields!("all workers stopped", status = "ok");
}

struct WorkerStatusTrack {
    worker: WorkerService,
    jobs: Vec<(String, Arc<AtomicBool>)>,
}

fn collect_status_tracks(handles: &[(WorkerService, Vec<JobHandle>)]) -> Vec<WorkerStatusTrack> {
    handles
        .iter()
        .map(|(worker, jobs)| WorkerStatusTrack {
            worker: *worker,
            jobs: jobs.iter().map(|job| (job.name().to_string(), job.status_flag())).collect(),
        })
        .collect()
}

fn log_pending_workers(tracks: &[WorkerStatusTrack], message: &str) {
    for track in tracks {
        let pending: Vec<_> = track
            .jobs
            .iter()
            .filter_map(|(name, flag)| if flag.load(Ordering::Relaxed) { None } else { Some(name.clone()) })
            .collect();
        if pending.is_empty() {
            continue;
        }
        info_with_fields!(message, worker = track.worker.as_ref(), jobs = pending.join(", "));
    }
}

async fn run_consumer_services(settings: settings::Settings, services: &[ConsumerService]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if services.is_empty() {
        info_with_fields!("no consumer services requested", status = "ok");
        return Ok(());
    }

    let settings = Arc::new(settings);
    let (shutdown_tx, shutdown_rx) = shutdown::channel();
    let signal_handle = shutdown::spawn_signal_handler(shutdown_tx);

    let consumer_metrics = Arc::new(metrics::consumer::ConsumerMetrics::new());
    let price_metrics = Arc::new(metrics::price::PriceMetrics::new());
    let composite = Arc::new(metrics::Metrics::new(vec![consumer_metrics.clone(), price_metrics.clone()]));
    let health_state = health::spawn_server(composite);
    let reporter: Arc<dyn ConsumerStatusReporter> = Arc::new(ConsumerReporter::new(consumer_metrics));
    let failures = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = services
        .iter()
        .map(|service| {
            let svc = service.clone();
            let svc_name = svc.as_ref().to_string();
            let settings = settings.clone();
            let reporter = reporter.clone();
            let shutdown_rx = shutdown_rx.clone();
            let failures = failures.clone();
            let price_metrics = price_metrics.clone();
            tokio::spawn(async move {
                match run_consumer((*settings.as_ref()).clone(), svc, shutdown_rx, reporter, price_metrics).await {
                    Ok(_) => info_with_fields!("consumer stopped", consumer = svc_name.as_str(), status = "ok"),
                    Err(err) => {
                        let message = err.to_string();
                        error_with_fields!("consumer failed", &*err, consumer = svc_name.as_str());
                        if let Ok(mut list) = failures.lock() {
                            list.push(format!("{}: {}", svc_name, message));
                        }
                    }
                }
            })
        })
        .collect();

    health_state.set_ready();

    signal_handle.await.ok();
    futures::future::join_all(handles).await;

    match failures.lock() {
        Ok(errors) if errors.is_empty() => {
            info_with_fields!("all consumers stopped", status = "ok");
            Ok(())
        }
        Ok(errors) => Err(errors.join(", ").into()),
        Err(_) => Err("failed to inspect consumer results".into()),
    }
}

async fn run_consumer(
    settings: settings::Settings,
    service: ConsumerService,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
    price_metrics: Arc<metrics::price::PriceMetrics>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match service {
        ConsumerService::Store => consumers::run_consumer_store(settings, shutdown_rx, reporter).await,
        ConsumerService::Indexer => consumers::run_consumer_indexer(settings, shutdown_rx, reporter).await,
        ConsumerService::Notifications => consumers::notifications::run(settings, shutdown_rx, reporter).await,
        ConsumerService::Rewards => consumers::run_consumer_rewards(settings, shutdown_rx, reporter).await,
        ConsumerService::Support => consumers::run_consumer_support(settings, shutdown_rx, reporter).await,
        ConsumerService::Fiat => consumers::run_consumer_fiat(settings, shutdown_rx, reporter).await,
        ConsumerService::Prices => consumers::run_consumer_prices(settings, shutdown_rx, reporter, price_metrics).await,
        ConsumerService::Assets => consumers::run_consumer_assets(settings, shutdown_rx, reporter).await,
    }
}
