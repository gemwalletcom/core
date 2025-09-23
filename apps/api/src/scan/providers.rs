use gem_client::ReqwestClient;
use security_provider::ScanProvider;
use security_provider::providers::goplus::GoPlusProvider;
use security_provider::providers::hashdit::HashDitProvider;
use settings::Settings;

pub struct ScanProviderFactory {}

impl ScanProviderFactory {
    pub fn create_providers(settings: &Settings) -> Vec<Box<dyn ScanProvider + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(settings.scan.timeout_ms))
            .build()
            .unwrap();

        vec![
            Box::new(GoPlusProvider::new(
                ReqwestClient::new(settings.scan.goplus.url.clone(), client.clone()),
                &settings.scan.goplus.key.public,
            )),
            Box::new(HashDitProvider::new(
                ReqwestClient::new(settings.scan.hashdit.url.clone(), client.clone()),
                &settings.scan.hashdit.key.public,
                &settings.scan.hashdit.key.secret,
            )),
        ]
    }
}
