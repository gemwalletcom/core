use security_goplus::GoPlusProvider;
use security_hashdit::HashDitProvider;
use security_provider::SecurityProvider;
use settings::Settings;

pub struct SecurityProviderFactory {}

impl SecurityProviderFactory {
    pub fn create_providers(settings: &Settings) -> Vec<Box<dyn SecurityProvider + Send + Sync>> {
        vec![
            Box::new(GoPlusProvider::new(&settings.security.goplus.url, &settings.security.goplus.api_id)),
            Box::new(HashDitProvider::new(&settings.security.hashdit.api_id, &settings.security.hashdit.api_secret)),
        ]
    }
}
