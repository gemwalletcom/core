use oneinch::OneInchClient;
use primitives::{Chain, SwapQuote, SwapQuoteProtocolRequest};
use swap_skip_client::client::SkipApi;

use crate::{JupiterClient, ThorchainSwapClient};

pub struct SwapperClient {
    oneinch: OneInchClient,
    jupiter: JupiterClient,
    thorchain: ThorchainSwapClient,
    skip: SkipApi,
}

impl SwapperClient {
    pub fn new(
        oneinch: OneInchClient,
        jupiter: JupiterClient,
        thorchain: ThorchainSwapClient,
        skip: SkipApi,
    ) -> Self {
        Self {
            oneinch,
            jupiter,
            thorchain,
            skip,
        }
    }

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        // Need to fetch quote from different providers and return the best one
        match quote.from_asset.chain {
            Chain::Ethereum
            | Chain::SmartChain
            | Chain::Optimism
            | Chain::Arbitrum
            | Chain::Polygon
            | Chain::Base
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::AvalancheC
            | Chain::Manta
            | Chain::Blast => self.oneinch.get_quote(quote).await,
            Chain::Solana => self.jupiter.get_quote(quote).await,
            Chain::Thorchain | Chain::Doge | Chain::Cosmos | Chain::Bitcoin | Chain::Litecoin => {
                self.thorchain.get_quote(quote).await
            }
            Chain::Osmosis | Chain::Celestia | Chain::Injective | Chain::Noble | Chain::Sei => {
                self.skip.get_quote(quote).await
            }
            Chain::Binance
            | Chain::Ton
            | Chain::Tron
            | Chain::Aptos
            | Chain::Sui
            | Chain::Xrp
            | Chain::OpBNB => todo!(),
        }
    }
}
