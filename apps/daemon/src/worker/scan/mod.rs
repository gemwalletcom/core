mod validator_scanner;

use job_runner::{JobStatusReporter, ShutdownReceiver, run_job};
use primitives::ConfigKey;
use settings::{Settings, service_user_agent};
use settings_chain::ChainProviders;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use tokio::task::JoinHandle;
use validator_scanner::ValidatorScanner;

pub async fn jobs(settings: Settings, reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());

    let update_validators = tokio::spawn(run_job(
        "update_chain_validators",
        config.get_duration(ConfigKey::ScanTimerUpdateValidators)?,
        reporter.clone(),
        shutdown_rx.clone(),
        {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let validator_scanner = ValidatorScanner::new(
                    ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_validators"))),
                    database.clone(),
                    &settings.assets.url,
                );
                async move { validator_scanner.update_validators("update_chain_validators").await }
            }
        },
    ));

    let update_validators_static_assets = tokio::spawn(run_job(
        "update_validators_from_static_assets",
        config.get_duration(ConfigKey::ScanTimerUpdateValidatorsStatic)?,
        reporter.clone(),
        shutdown_rx,
        {
            let settings = settings.clone();
            let database = database.clone();
            move || {
                let validator_scanner = ValidatorScanner::new(
                    ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_static_assets"))),
                    database.clone(),
                    &settings.assets.url,
                );
                async move { validator_scanner.update_validators_from_static_assets("update_validators_from_static_assets").await }
            }
        },
    ));

    Ok(vec![update_validators, update_validators_static_assets])
}
