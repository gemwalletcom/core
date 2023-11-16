use primitives::{SwapQuoteProtocolRequest, SwapQuote};

use crate::oneinch::OneInchClient;

pub struct SwapperClient {
    oneinch: OneInchClient,
}

impl SwapperClient {
    pub fn new(
        oneinch: OneInchClient,
    ) -> Self {
        Self {oneinch}
    }  

    pub async fn get_quote(&self, quote: SwapQuoteProtocolRequest) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        return self.oneinch.get_quote(quote).await;
    }
}