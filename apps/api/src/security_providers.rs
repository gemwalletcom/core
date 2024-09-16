use security_goplus::GoPlusClient;
use security_hashdit::HashDitClient;
use security_provider::SecurityProvider;
use settings::Settings;

pub struct SecurityProviderFactory {}

impl SecurityProviderFactory {
    pub fn create_providers(settings: &Settings) -> Vec<Box<dyn SecurityProvider + Send + Sync>> {
        vec![
            Box::new(GoPlusClient::new(&settings.security.goplus.url, &settings.security.goplus.api_key)),
            Box::new(HashDitClient::new(&settings.security.hashdit.url, &settings.security.hashdit.api_key)),
        ]
    }
}
