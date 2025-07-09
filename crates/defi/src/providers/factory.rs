use std::collections::HashMap;
use std::sync::Arc;

use primitives::Chain;

use crate::provider::DeFiProvider;
use crate::providers::debank::DeBankProvider;

pub struct DefiProviderFactory {
    providers: HashMap<String, Arc<dyn DeFiProvider>>,
}

impl DefiProviderFactory {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn with_debank(mut self, api_key: String) -> Self {
        let provider = Arc::new(DeBankProvider::new(api_key));
        self.providers.insert("debank".to_string(), provider);
        self
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn DeFiProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn get_all_providers(&self) -> Vec<Arc<dyn DeFiProvider>> {
        self.providers.values().cloned().collect()
    }

    pub fn get_providers_for_chain(&self, chain: &Chain) -> Vec<Arc<dyn DeFiProvider>> {
        self.providers
            .values()
            .filter(|provider| provider.supported_chain_types().contains(&chain.chain_type()))
            .cloned()
            .collect()
    }
}

impl Default for DefiProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}
