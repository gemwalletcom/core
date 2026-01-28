mod validator_scanner;

use job_runner::{ShutdownReceiver, run_job};
use primitives::ConfigKey;
use settings::{Settings, service_user_agent};
use settings_chain::ChainProviders;
use std::error::Error;
use storage::ConfigCacher;
use tokio::task::JoinHandle;
use validator_scanner::ValidatorScanner;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());

    let update_validators = tokio::spawn(run_job(
        "Update chain validators",
        config.get_duration(ConfigKey::ScanTimerUpdateValidators)?,
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
                async move { validator_scanner.update_validators("Update chain validators").await }
            }
        },
    ));

    let update_validators_static_assets = tokio::spawn(run_job(
        "Update validators from static assets",
        config.get_duration(ConfigKey::ScanTimerUpdateValidatorsStatic)?,
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
                async move { validator_scanner.update_validators_from_static_assets("Update validators from static assets").await }
            }
        },
    ));

    Ok(vec![update_validators, update_validators_static_assets])
}
