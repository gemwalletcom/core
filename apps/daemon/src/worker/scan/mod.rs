mod validator_scanner;

use job_runner::run_job;
use settings::{Settings, service_user_agent};
use settings_chain::ChainProviders;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use validator_scanner::ValidatorScanner;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);

    let update_validators = run_job("Update chain validators", Duration::from_secs(86400), {
        let settings = settings.clone();
        let database = database.clone();
        move || {
            let mut validator_scanner = ValidatorScanner::new(
                ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_validators"))),
                database.clone(),
                &settings.assets.url,
            );
            async move { validator_scanner.update_validators("Update chain validators").await }
        }
    });

    let update_validators_static_assets = run_job("Update validators from static assets", Duration::from_secs(3600), {
        let settings = settings.clone();
        let database = database.clone();
        move || {
            let mut validator_scanner = ValidatorScanner::new(
                ChainProviders::from_settings(&settings, &service_user_agent("daemon", Some("scan_static_assets"))),
                database.clone(),
                &settings.assets.url,
            );
            async move {
                validator_scanner
                    .update_validators_from_static_assets("Update validators from static assets")
                    .await
            }
        }
    });

    vec![Box::pin(update_validators), Box::pin(update_validators_static_assets)]
}
