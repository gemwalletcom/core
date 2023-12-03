use primitives::{SwapQuoteProtocolRequest, SwapQuote, Chain};

use crate::{OneInchClient, JupiterClient, ThorchainSwapClient};

pub struct SwapperClient {
    oneinch: OneInchClient,
    jupiter: JupiterClient,
    thorchain: ThorchainSwapClient,
}

impl SwapperClient {
    pub fn new(
        oneinch: OneInchClient,
        jupiter: JupiterClient,
        thorchain: ThorchainSwapClient,
    ) -> Self {
        Self {
            oneinch,
            jupiter,
            thorchain,
        }
    }  

    pub async fn get_quote(&self, quote: SwapQuoteProtocolRequest) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        match quote.from_asset.chain {
            Chain::Ethereum |
            Chain::SmartChain |
            Chain::Optimism |
            Chain::Arbitrum |
            Chain::Polygon |
            Chain::Base |
            Chain::Fantom |
            Chain::Gnosis |
            Chain::AvalancheC => {
                self.oneinch.get_quote(quote).await
            }
            Chain::Binance => todo!(),
            Chain::Solana => {
                self.jupiter.get_quote(quote).await
            }
            Chain::Thorchain |
            Chain::Doge |
            Chain::Cosmos | 
            Chain::Bitcoin |
            Chain::Litecoin => {
                self.thorchain.get_quote(quote).await
            },
            Chain::Osmosis => todo!(),
            Chain::Celestia => todo!(),
            Chain::Ton => todo!(),
            Chain::Tron => todo!(),
            Chain::Aptos => todo!(),
            Chain::Sui => todo!(),
            Chain::Ripple => todo!(),
            Chain::OpBNB => todo!(),
        }
    }
}