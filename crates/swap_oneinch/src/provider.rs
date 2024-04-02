use crate::client::{OneInchClient, PROVIDER_NAME};
use async_trait::async_trait;
use primitives::{SwapQuote, SwapQuoteProtocolRequest};
use swap_provider::{SwapError, SwapProvider};

pub struct OneInchProvider {
    pub client: OneInchClient,
}

#[async_trait]
impl SwapProvider for OneInchProvider {
    fn provider(&self) -> primitives::SwapProvider {
        PROVIDER_NAME.into()
    }

    fn supported_chains(&self) -> Vec<primitives::Chain> {
        self.client.chains()
    }

    async fn get_quote(&self, request: SwapQuoteProtocolRequest) -> Result<SwapQuote, SwapError> {
        let quote = self.client.get_quote(request).await?;
        Ok(quote)
    }
}
