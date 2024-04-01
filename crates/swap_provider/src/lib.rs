use async_trait::async_trait;

pub const SWAP_SLIPPAGE: f32 = 0.01;
pub type SwapError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait SwapProvider {
    fn provider(&self) -> primitives::SwapProvider;
    async fn get_quote(
        &self,
        request: primitives::SwapQuoteProtocolRequest,
    ) -> Result<primitives::SwapQuote, SwapError>;
}
