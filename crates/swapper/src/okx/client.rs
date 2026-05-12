use super::{
    auth::{build_headers, build_query_string},
    model::{OkxApiResponse, OkxClientConfig, QuoteData, QuoteParams, SwapDataResult, SwapParams},
};
use crate::SwapperError;
use chrono::{SecondsFormat, Utc};
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub(super) struct OkxDexClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
    config: OkxClientConfig,
}

impl<C> OkxDexClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C, config: OkxClientConfig) -> Self {
        Self { client, config }
    }

    pub async fn get_quote(&self, params: &QuoteParams) -> Result<OkxApiResponse<QuoteData>, SwapperError> {
        self.signed_get("/api/v6/dex/aggregator/quote", params).await
    }

    pub async fn get_swap_data(&self, params: &SwapParams) -> Result<OkxApiResponse<SwapDataResult>, SwapperError> {
        self.signed_get("/api/v6/dex/aggregator/swap", params).await
    }

    async fn signed_get<P, R>(&self, path: &str, params: &P) -> Result<R, SwapperError>
    where
        P: serde::Serialize,
        R: serde::de::DeserializeOwned + Send,
    {
        let query = build_query_string(params)?;
        let full_path = format!("{path}{query}");
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let headers = build_headers(&self.config, &timestamp, &full_path);
        self.client.get_with_headers(&full_path, headers).await.map_err(SwapperError::from)
    }
}
