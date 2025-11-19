use cacher::CacherClient;
use primitives::FiatQuoteData;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFiatQuoteData {
    pub quote: FiatQuoteData,
    #[serde(flatten)]
    pub asset_symbol: primitives::FiatAssetSymbol,
}

pub struct FiatCacherClient {
    cacher: CacherClient,
}

impl FiatCacherClient {
    const QUOTE_TTL_SECONDS: i64 = 15 * 60;

    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    fn quote_cache_key(quote_id: &str) -> String {
        format!("fiat_quote:{}", quote_id)
    }

    pub async fn set_quotes(&self, cached_quotes: &[CachedFiatQuoteData]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ttl_seconds = Self::QUOTE_TTL_SECONDS;
        let values: Vec<_> = cached_quotes.iter().map(|x| (Self::quote_cache_key(&x.quote.id), x)).collect();
        let entries: Vec<_> = values.iter().map(|(key, value)| (key.as_str(), *value)).collect();

        self.cacher.set_values_with_ttl(entries, ttl_seconds).await
    }

    pub async fn get_quote(&self, quote_id: &str) -> Result<CachedFiatQuoteData, Box<dyn Error + Send + Sync>> {
        let key = Self::quote_cache_key(quote_id);
        self.cacher.get_value(&key).await
    }
}
