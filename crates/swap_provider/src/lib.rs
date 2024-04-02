use async_trait::async_trait;

pub const DEFAULT_SWAP_SLIPPAGE: f32 = 0.01;
pub type ProviderList = Vec<Box<dyn SwapProvider + Send + Sync>>;
pub type SwapError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait SwapProvider {
    fn provider(&self) -> primitives::SwapProvider;
    fn supported_chains(&self) -> Vec<primitives::Chain>;
    async fn get_quote(
        &self,
        request: primitives::SwapQuoteProtocolRequest,
    ) -> Result<primitives::SwapQuote, SwapError>;
}
