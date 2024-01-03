use primitives::{Chain, SwapQuote, SwapQuoteProtocolRequest};

use crate::{JupiterClient, OneInchClient, ThorchainSwapClient};

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

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        match quote.from_asset.chain {
            Chain::Ethereum
            | Chain::SmartChain
            | Chain::Optimism
            | Chain::Arbitrum
            | Chain::Polygon
            | Chain::Base
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::AvalancheC => self.oneinch.get_quote(quote).await,
            Chain::Solana => self.jupiter.get_quote(quote).await,
            Chain::Thorchain | Chain::Doge | Chain::Cosmos | Chain::Bitcoin | Chain::Litecoin => {
                self.thorchain.get_quote(quote).await
            }
            Chain::Osmosis
            | Chain::Celestia
            | Chain::Binance
            | Chain::Injective
            | Chain::Ton
            | Chain::Tron
            | Chain::Aptos
            | Chain::Sui
            | Chain::Xrp
            | Chain::OpBNB
            | Chain::Sei => todo!(),
        }
    }
}
