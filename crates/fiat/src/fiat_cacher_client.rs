use cacher::{CacheError, CacheKey, CacherClient};
use primitives::{FiatAssetSymbol, FiatQuote};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFiatQuoteData {
    pub quote: FiatQuote,
    #[serde(flatten)]
    pub asset_symbol: FiatAssetSymbol,
    #[serde(default)]
    pub country_code: Option<String>,
}

pub struct FiatCacherClient {
    cacher: CacherClient,
}

impl FiatCacherClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn set_quotes(&self, cached_quotes: &[CachedFiatQuoteData]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let entries: Vec<_> = cached_quotes.iter().map(|x| (CacheKey::FiatQuote(&x.quote.id), x)).collect();
        self.cacher.set_values_cached(&entries).await
    }

    pub async fn get_quote(&self, quote_id: &str) -> Result<CachedFiatQuoteData, Box<dyn Error + Send + Sync>> {
        match self.cacher.get_cached_optional(CacheKey::FiatQuote(quote_id)).await? {
            Some(quote) => Ok(quote),
            None => Err(Box::new(CacheError::not_found("FiatQuote", quote_id.to_string()))),
        }
    }
}
