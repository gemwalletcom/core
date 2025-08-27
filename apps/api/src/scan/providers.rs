use security_provider::providers::goplus::GoPlusProvider;
use security_provider::providers::hashdit::HashDitProvider;
use security_provider::ScanProvider;
use settings::Settings;

pub struct ScanProviderFactory {}

impl ScanProviderFactory {
    pub fn create_providers(settings: &Settings) -> Vec<Box<dyn ScanProvider + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(settings.scan.timeout_ms))
            .build()
            .unwrap();

        vec![
            Box::new(GoPlusProvider::new(client.clone(), &settings.scan.goplus.url, &settings.scan.goplus.key.public)),
            Box::new(HashDitProvider::new(
                client.clone(),
                &settings.scan.hashdit.key.public,
                &settings.scan.hashdit.key.secret,
            )),
        ]
    }
}
