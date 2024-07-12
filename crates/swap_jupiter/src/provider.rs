use crate::client::{JupiterClient, JUPITER};
use async_trait::async_trait;
use primitives::{Chain, SwapQuote, SwapQuoteProtocolRequest};
use swap_provider::{SwapError, SwapProvider};

pub struct JupiterProvider {
    pub client: JupiterClient,
}

#[async_trait]
impl SwapProvider for JupiterProvider {
    fn provider(&self) -> primitives::SwapProvider {
        JUPITER.into()
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }

    async fn get_quote(&self, request: SwapQuoteProtocolRequest) -> Result<SwapQuote, SwapError> {
        let quote = self.client.get_quote(request).await?;
        Ok(quote)
    }
}
