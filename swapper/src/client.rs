use primitives::{SwapQuoteProtocolRequest, SwapQuote, Chain};

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
        match quote.from_asset.chain {
            Chain::Ethereum |
            Chain::SmartChain |
            Chain::Optimism |
            Chain::Arbitrum |
            Chain::Polygon |
            Chain::Base |
            Chain::AvalancheC => {
                return self.oneinch.get_quote(quote).await;
            }
            primitives::Chain::Bitcoin => todo!(),
            primitives::Chain::Litecoin => todo!(),
            primitives::Chain::Binance => todo!(),
            primitives::Chain::Solana => todo!(),
            primitives::Chain::Thorchain => todo!(),
            primitives::Chain::Cosmos => todo!(),
            primitives::Chain::Osmosis => todo!(),
            primitives::Chain::Ton => todo!(),
            primitives::Chain::Tron => todo!(),
            primitives::Chain::Doge => todo!(),
            primitives::Chain::Aptos => todo!(),
            primitives::Chain::Sui => todo!(),
            primitives::Chain::Ripple => todo!(),
            primitives::Chain::OpBNB => todo!(),
        }
    }
}